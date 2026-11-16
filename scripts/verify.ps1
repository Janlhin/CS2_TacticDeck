#requires -Version 7.4
<#
.SYNOPSIS
    唯一验收入口：文档结构、字段、链接、根目录白名单、脚本基线与代码校验。

.DESCRIPTION
    任一步骤失败即整体失败（退出码 1）。禁止通过 --no-verify、注释检查或放宽规则等方式绕过。

.PARAMETER NoCargo
    跳过 Rust 相关步骤（无 Cargo.toml 时自动跳过）。

.PARAMETER NoExternal
    跳过外链可达性检查（离线环境或网络不稳定时使用，须在交付说明中标注「未验证」）。

.PARAMETER NoDocCommands
    跳过 README 命令块实跑。
#>
[CmdletBinding()]
param(
    [switch]$NoCargo,
    [switch]$NoExternal,
    [switch]$NoDocCommands
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$script:Failures = @()
$script:Steps = @()

function Write-Step([string]$Name, [string]$Status, [string]$Detail = '') {
    $script:Steps += [pscustomobject]@{ Name = $Name; Status = $Status; Detail = $Detail }
    $line = '[{0}] {1}' -f $Status.PadRight(6), $Name
    if ($Detail) { $line += " — $Detail" }
    Write-Host $line
}

function Invoke-Step([string]$Name, [scriptblock]$Body) {
    Write-Step $Name 'RUN' ''
    try {
        $result = & $Body
        if ($result -is [array]) { $detail = ($result -join '; ') } else { $detail = [string]$result }
        Write-Step $Name 'PASS' $detail
    }
    catch {
        $script:Failures += "$Name : $($_.Exception.Message)"
        Write-Step $Name 'FAIL' $_.Exception.Message
    }
}

function Fail([string]$Message) { throw $Message }

# 已跟踪 + 未跟踪（排除 .gitignore）——只看 ls-files 会漏掉尚未 git add 的散落文件
function Get-RepoFiles {
    @(& git -C $repo ls-files --cached --others --exclude-standard)
}

# ---------------------------------------------------------------- 1. 五个文档
Invoke-Step 'docs/five-files' {
    $required = @('README.md', 'AGENTS.md', 'DECISIONS.md', 'TASKS.md', 'MANIFEST.md')
    $missing = $required | Where-Object { -not (Test-Path -LiteralPath (Join-Path $repo $_)) }
    if ($missing) { Fail "缺少文档: $($missing -join ', ')" }
    $ag = (Get-Content -LiteralPath (Join-Path $repo 'AGENTS.md') | Measure-Object -Line).Lines
    if ($ag -gt 150) { Fail "AGENTS.md 超过 150 行（当前 $ag）" }
    "5 个文档齐；AGENTS.md $ag 行"
}

# ---------------------------------------------------------------- 2. README 字段
Invoke-Step 'docs/readme-fields' {
    $readme = Get-Content -LiteralPath (Join-Path $repo 'README.md') -Raw
    $fields = @(
        '## 它解决什么',
        '## 验收标准（可机器判定）',
        '## 技术栈与版本',
        '## 目录结构',
        '## 安装与运行',
        '## 已知限制'
    )
    $miss = @()
    foreach ($f in $fields) { if ($readme -notmatch [regex]::Escape($f)) { $miss += $f } }
    if ($miss) { Fail "README 缺少小节: $($miss -join ', ')" }
    if ($readme -match '暂无|待定') { Fail "README 含占位词（暂无/待定），不计为非空" }
    "$($fields.Count) 个小节齐且无占位词"
}

# ---------------------------------------------------------------- 3. 台账字段
Invoke-Step 'docs/ledger-fields' {
    $decisions = Get-Content -LiteralPath (Join-Path $repo 'DECISIONS.md') -Raw
    $blocks = [regex]::Matches($decisions, '(?ms)^## (D-\d+).*?(?=^## |\z)')
    if ($blocks.Count -lt 1) { Fail 'DECISIONS.md 无任何条目' }
    $need = @('**日期**', '**决策**', '**背景**', '**被采纳方案**', '**被否决的替代方案与理由**', '**影响范围**', '**复审条件**')
    foreach ($b in $blocks) {
        foreach ($n in $need) { if ($b.Value -notmatch [regex]::Escape($n)) { Fail "条目 $($b.Groups[1].Value) 缺少字段 $n" } }
    }
    $tasks = Get-Content -LiteralPath (Join-Path $repo 'TASKS.md') -Raw
    if ($tasks -notmatch '## 待裁决清单') { Fail 'TASKS.md 缺少「待裁决清单」' }
    if ($tasks -notmatch '期望退出码|→ 0') { Fail 'TASKS.md 条目缺少可机器判定的完成判据' }
    $manifest = Get-Content -LiteralPath (Join-Path $repo 'MANIFEST.md') -Raw
    if ($manifest -notmatch 'SHA256') { Fail 'MANIFEST.md 缺少 SHA256 字段' }
    if ($manifest -notmatch '许可结论') { Fail 'MANIFEST.md 缺少许可结论字段' }
    "$($blocks.Count) 条决策；任务与登记字段齐"
}

# ---------------------------------------------------------------- 4. 根目录白名单
Invoke-Step 'layout/root-whitelist' {
    $tracked = Get-RepoFiles
    if ($LASTEXITCODE -ne 0) { Fail 'git ls-files 失败（是否在 Git 仓库内？）' }
    $top = $tracked | ForEach-Object { ($_ -split '/')[0] } | Sort-Object -Unique
    $allowed = @(
        'README.md', 'AGENTS.md', 'DECISIONS.md', 'TASKS.md', 'MANIFEST.md',
        'LICENSE', 'CHANGELOG.md',
        'package.json', 'package-lock.json',
        '.gitignore', '.gitattributes', '.editorconfig', '.npmrc', '.nvmrc', '.env.example',
        'commitlint.config.mjs', 'tsconfig.json', 'eslint.config.mjs', 'vitest.config.mjs', 'playwright.config.mjs',
        '.github', '.husky', '.vscode', 'CODEOWNERS',
        'index.html', 'vite.config.ts', 'tsconfig.json', 'sass.config.json',
        'src', 'src-tauri',
        'docs', 'scripts', 'config', 'crates', 'apps'
    )
    $extra = @($top | Where-Object { $_ -notin $allowed })
    if ($extra.Count -gt 0) { Fail "根目录存在白名单外条目: $($extra -join ', ')" }
    "顶层 $($top.Count) 项，全部在白名单内"
}

# ---------------------------------------------------------------- 5. 脚本基线
Invoke-Step 'scripts/baseline' {
    $bad = (Get-RepoFiles) | Where-Object { $_ -match '\.(cmd|bat)$' }
    if ($bad) { Fail "存在 .cmd / .bat: $($bad -join ', ')" }
    # 用拼接构造模式，避免本文件自身的检测规则被匹配到
    $pat = 'cmd' + [char]46 + 'exe|powershell' + [char]46 + 'exe'
    $hits = Select-String -Path (Join-Path $repo 'scripts/*.ps1'), (Join-Path $repo 'scripts/hooks/*.ps1') -Pattern $pat -ErrorAction SilentlyContinue
    if ($hits) { Fail "脚本中出现 cmd / powershell 调用: $($hits[0].Path):$($hits[0].LineNumber)" }
    '0 个 .cmd/.bat；脚本未调用 cmd / powershell'
}

# ---------------------------------------------------------------- 6. 文档格式
Invoke-Step 'docs/format' {
    $md = (Get-RepoFiles) | Where-Object { $_ -match '\.md$' }
    $pairs = $md | Where-Object { Test-Path -LiteralPath (Join-Path $repo ($_ -replace '\.md$', '.adoc')) }
    if ($pairs) { Fail "存在同名 .adoc 并存: $($pairs -join ', ')" }
    "$($md.Count) 个 Markdown 文件，无 .adoc 并存"
}

# ---------------------------------------------------------------- 7. 文档 lint
Invoke-Step 'docs/lint' {
    $bin = Join-Path $repo 'node_modules/.bin/markdownlint-cli2.cmd'
    if (-not (Test-Path -LiteralPath $bin)) { Fail '未安装 markdownlint-cli2（先运行 npm ci）' }
    & $bin --config (Join-Path $repo 'config/markdownlint-cli2.jsonc') 'README.md' 'AGENTS.md' 'DECISIONS.md' 'TASKS.md' 'MANIFEST.md' 'docs/*.md'
    if ($LASTEXITCODE -ne 0) { Fail "markdownlint-cli2 退出码 $LASTEXITCODE" }
    '零告警'
}

# ---------------------------------------------------------------- 8. 内部链接与锚点
Invoke-Step 'docs/links-internal' {
    $bin = Join-Path $repo 'node_modules/.bin/remark.cmd'
    if (-not (Test-Path -LiteralPath $bin)) { Fail '未安装 remark-cli（先运行 npm ci）' }
    $targets = @('README.md', 'AGENTS.md', 'DECISIONS.md', 'TASKS.md', 'MANIFEST.md', 'docs/engineering-standards.md', 'docs/third-party-notices.md')
    $out = & $bin --frail --quiet --use remark-validate-links @targets 2>&1
    $text = ($out | Out-String).Trim()
    # remark-cli 在部分环境下即使有 warning 也返回 0，因此以输出内容为准
    if ($LASTEXITCODE -ne 0 -or $text -match 'missing-heading|Cannot find|missing-file') {
        Fail "锚点或相对链接失效：$text"
    }
    '锚点与相对链接全部可达'
}

# ---------------------------------------------------------------- 9. 外链
if (-not $NoExternal) {
    Invoke-Step 'docs/links-external' {
        $bin = Join-Path $repo 'node_modules/.bin/markdown-link-check.cmd'
        if (-not (Test-Path -LiteralPath $bin)) { Fail '未安装 markdown-link-check（先运行 npm ci）' }
        & $bin --config (Join-Path $repo 'config/markdown-link-check.json') 'README.md' 'docs/third-party-notices.md'
        if ($LASTEXITCODE -ne 0) { Fail "markdown-link-check 退出码 $LASTEXITCODE" }
        '外链可达'
    }
}
else { Write-Step 'docs/links-external' 'SKIP' '按参数跳过（交付说明须标注未验证）' }

# ---------------------------------------------------------------- 10. README 命令块
if (-not $NoDocCommands) {
    Invoke-Step 'docs/commands' {
        $readme = Get-Content -LiteralPath (Join-Path $repo 'README.md') -Raw
        $blocks = [regex]::Matches($readme, '(?ms)```powershell\r?\n(.*?)```')
        if ($blocks.Count -eq 0) { Fail 'README 无 powershell 代码块' }
        foreach ($b in $blocks) {
            $cmd = $b.Groups[1].Value.Trim()
            if ($cmd -match 'npm run verify') { continue }   # 递归入口，由外层 verify 自身保证
            Push-Location -LiteralPath $repo
            try {
                $out = & pwsh -NoLogo -NoProfile -Command $cmd 2>&1
                if ($LASTEXITCODE -ne 0) { Fail "命令块失败: $cmd" }
            }
            finally { Pop-Location }
        }
        "$($blocks.Count) 个命令块可执行"
    }
}
else { Write-Step 'docs/commands' 'SKIP' '按参数跳过' }

# ---------------------------------------------------------------- 11. SemVer
Invoke-Step 'version/semver' {
    $pkgPath = Join-Path $repo 'package.json'
    if (-not (Test-Path -LiteralPath $pkgPath)) { Fail '缺少 package.json' }
    $v = (Get-Content -LiteralPath $pkgPath -Raw | ConvertFrom-Json).version
    $re = '^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*)(?:\.(?:0|[1-9]\d*|\d*[a-zA-Z-][0-9a-zA-Z-]*))*)?(?:\+[0-9a-zA-Z-]+(?:\.[0-9a-zA-Z-]+)*)?$'
    if ($v -notmatch $re) { Fail "版本号不合法: $v" }
    "版本 $v"
}

# ---------------------------------------------------------------- 12. Rust
$cargoManifest = Join-Path $repo 'src-tauri/Cargo.toml'
$hasCargo = Test-Path -LiteralPath $cargoManifest
if ($hasCargo -and -not $NoCargo) {
    Invoke-Step 'rust/fmt' {
        & cargo fmt --manifest-path $cargoManifest --all -- --check
        if ($LASTEXITCODE -ne 0) { Fail "cargo fmt 退出码 $LASTEXITCODE" }
        '格式一致'
    }
    Invoke-Step 'rust/clippy' {
        & cargo clippy --manifest-path $cargoManifest --all-targets -- -D warnings
        if ($LASTEXITCODE -ne 0) { Fail "cargo clippy 退出码 $LASTEXITCODE" }
        '零警告'
    }
    Invoke-Step 'rust/test' {
        & cargo test --manifest-path $cargoManifest --all
        if ($LASTEXITCODE -ne 0) { Fail "cargo test 退出码 $LASTEXITCODE" }
        '全部通过'
    }
}
else {
    Write-Step 'rust/fmt' 'SKIP' '无 src-tauri/Cargo.toml 或按参数跳过'
    Write-Step 'rust/clippy' 'SKIP' '同上'
    Write-Step 'rust/test' 'SKIP' '同上'
}

# ---------------------------------------------------------------- 汇总
Write-Host ''
Write-Host ('=' * 62)
if ($script:Failures.Count -gt 0) {
    Write-Host "FAIL  共 $($script:Failures.Count) 项失败："
    $script:Failures | ForEach-Object { Write-Host "  - $_" }
    exit 1
}
Write-Host 'PASS  全部检查通过（唯一验收入口 npm run verify）'
exit 0
