#requires -Version 7.4
<#
.SYNOPSIS
    测试入口：运行 Rust 单元测试。
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo 'src-tauri/Cargo.toml'

if (-not (Test-Path -LiteralPath $manifest)) {
    Write-Host '[SKIP]  无 src-tauri/Cargo.toml，测试阶段尚未开始'
    exit 0
}

& cargo test --manifest-path $manifest --all
exit $LASTEXITCODE
