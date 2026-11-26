#requires -Version 7.4
<#
.SYNOPSIS
    冷启动计时与断言：拉起 release 产物，断言每次窗口出现都快于阈值。

.DESCRIPTION
    冷启动口径（与 README「已知限制」一致）：
      - 被测对象是打包产出的 release 可执行文件（与 NSIS 安装包里打的是同一个 exe），
        不是 debug 产物，也不是浏览器里的前端。
      - 每次测量前确保同名进程未在运行，再首次拉起；统计的是「进程启动 → 主窗口出现」
        的耗时，不做任何预热，也不挑最好的一次。
      - 本机无管理员权限，不强制清空系统文件缓存（standby list）；因此本口径接近
        "第二次以后的本机冷启动"，与真正的开机首启（文件缓存全冷）存在差异，
        该差异已在 README「已知限制」如实标注。

    断言：每一轮测量都必须小于 -ThresholdMs；任一轮超时或超限即整体失败（退出码 1）。

.PARAMETER Exe
    被测可执行文件路径。缺省时按 cargo metadata 的 target_directory 推导
    `<target_directory>/release/cs2-tacticdeck.exe`，不在脚本里写死共享缓存路径。

.PARAMETER ThresholdMs
    每一轮允许的最长启动耗时（毫秒），默认 3000。

.PARAMETER Runs
    重复测量轮数，默认 3。每轮独立计时，不预热、不取最优值。

.PARAMETER TimeoutMs
    单轮等待窗口出现的超时（毫秒），超过即判失败，默认 10000。

.EXAMPLE
    pwsh -NoLogo -NoProfile -File scripts/cold-start.ps1
    pwsh -NoLogo -NoProfile -File scripts/cold-start.ps1 -ThresholdMs 3000 -Runs 3
#>
[CmdletBinding()]
param(
    [string]$Exe,
    [int]$ThresholdMs = 3000,
    [int]$Runs = 3,
    [int]$TimeoutMs = 10000
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot

function Resolve-ReleaseExe {
    param([string]$Candidate)

    if ($Candidate) {
        if (-not (Test-Path -LiteralPath $Candidate)) {
            throw "指定的 -Exe 不存在: $Candidate"
        }
        return (Resolve-Path -LiteralPath $Candidate).Path
    }

    $manifest = Join-Path $repo 'src-tauri/Cargo.toml'
    if (-not (Test-Path -LiteralPath $manifest)) {
        throw "未找到 src-tauri/Cargo.toml，且未通过 -Exe 指定可执行文件"
    }

    $metadata = & cargo metadata --manifest-path $manifest --format-version 1
    if ($LASTEXITCODE -ne 0) {
        throw "cargo metadata 失败（退出码 $LASTEXITCODE），无法推导 target_directory"
    }
    $meta = $metadata | ConvertFrom-Json
    $exePath = Join-Path $meta.target_directory 'release/cs2-tacticdeck.exe'
    if (-not (Test-Path -LiteralPath $exePath)) {
        throw "release 产物不存在: $exePath（先运行 bun run app:build）"
    }
    return (Resolve-Path -LiteralPath $exePath).Path
}

# 只结束路径与目标 exe 完全相同的实例，避免误杀别的进程。
function Stop-MatchingInstance {
    param([string]$TargetExe)

    $targetFull = [System.IO.Path]::GetFullPath($TargetExe)
    $procs = Get-Process -ErrorAction SilentlyContinue |
        Where-Object { $_.Path -and $_.Path.Equals($targetFull, [System.StringComparison]::OrdinalIgnoreCase) }
    foreach ($p in $procs) {
        Write-Host "[pre] 结束残留实例 PID=$($p.Id)"
        Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
    }
    if ($procs) { Start-Sleep -Milliseconds 500 }
}

function Wait-Window {
    param(
        [int]$ProcessId,
        [int]$TimeoutMs
    )

    $deadline = [Diagnostics.Stopwatch]::GetTimestamp() +
        [int64]([Math]::Ceiling($TimeoutMs * [Diagnostics.Stopwatch]::Frequency / 1000))
    while ([Diagnostics.Stopwatch]::GetTimestamp() -lt $deadline) {
        $proc = Get-Process -Id $ProcessId -ErrorAction SilentlyContinue
        if (-not $proc) { return $null }
        if ($proc.HasExited) { return 'exited' }
        if ($proc.MainWindowHandle -ne 0 -and $proc.MainWindowTitle -match 'CS2 TacticDeck') {
            return 'ready'
        }
        Start-Sleep -Milliseconds 100
    }
    return 'timeout'
}

$exePath = Resolve-ReleaseExe -Candidate $Exe
Write-Host "[info] 被测 exe: $exePath"
Write-Host "[info] 阈值 $ThresholdMs ms；轮数 $Runs；单轮超时 $TimeoutMs ms"

Stop-MatchingInstance -TargetExe $exePath

$measurements = @()
$failures = @()

for ($i = 1; $i -le $Runs; $i++) {
    Stop-MatchingInstance -TargetExe $exePath

    $watch = [Diagnostics.Stopwatch]::StartNew()
    $started = Start-Process -FilePath $exePath -PassThru
    $pid_ = $started.Id

    $result = Wait-Window -ProcessId $pid_ -TimeoutMs $TimeoutMs
    $watch.Stop()
    $ms = [int]$watch.Elapsed.TotalMilliseconds

    if ($result -ne 'ready') {
        $failures += "第 $i 轮：窗口未在 $TimeoutMs ms 内出现（$result）"
        Stop-Process -Id $pid_ -Force -ErrorAction SilentlyContinue
        break
    }

    $measurements += $ms
    Write-Host ("[run {0}/{1}] {2} ms" -f $i, $Runs, $ms)

    # 关闭本轮实例：先请求正常关闭，不退再强杀，避免下一轮被当成残留。
    $proc = Get-Process -Id $pid_ -ErrorAction SilentlyContinue
    if ($proc) {
        $null = $proc.CloseMainWindow()
        $proc.WaitForExit(3000) | Out-Null
        if (-not $proc.HasExited) {
            Stop-Process -Id $pid_ -Force -ErrorAction SilentlyContinue
        }
    }

    if ($ms -ge $ThresholdMs) {
        $failures += "第 $i 轮：$ms ms ≥ 阈值 $ThresholdMs ms"
        break
    }

    # 轮间留一点时间让进程与资源彻底退出，避免上一轮影响下一轮。
    Start-Sleep -Milliseconds 1500
}

Write-Host ''
if ($measurements.Count -gt 0) {
    $max = ($measurements | Measure-Object -Maximum).Maximum
    Write-Host ("各轮毫秒值: {0}" -f ($measurements -join ', '))
    Write-Host "最大值: $max ms（阈值 $ThresholdMs ms）"
}

if ($failures.Count -gt 0) {
    Write-Host ''
    Write-Host 'FAIL 冷启动断言未通过：'
    $failures | ForEach-Object { Write-Host "  - $_" }
    exit 1
}

Write-Host "PASS 冷启动 $Runs 轮全部 < $ThresholdMs ms"
exit 0
