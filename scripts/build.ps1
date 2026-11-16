#requires -Version 7.4
<#
.SYNOPSIS
    构建入口。当前仓库处于文档与门禁阶段，无 Cargo.toml 时打印 SKIP 并以 0 退出。
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot

if (-not (Test-Path -LiteralPath (Join-Path $repo 'Cargo.toml'))) {
    Write-Host '[SKIP]  无 Cargo.toml，构建阶段尚未开始（见 TASKS.md S0）'
    exit 0
}

& cargo build --all
exit $LASTEXITCODE
