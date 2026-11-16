#requires -Version 7.4
<#
.SYNOPSIS
    测试入口。当前无 Cargo.toml 时打印 SKIP 并以 0 退出；存在时运行 cargo test。
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot

if (-not (Test-Path -LiteralPath (Join-Path $repo 'Cargo.toml'))) {
    Write-Host '[SKIP]  无 Cargo.toml，测试阶段尚未开始（见 TASKS.md）'
    exit 0
}

& cargo test --all
exit $LASTEXITCODE
