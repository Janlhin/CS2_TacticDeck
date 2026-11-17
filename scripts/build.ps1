#requires -Version 7.4
<#
.SYNOPSIS
    构建入口：构建 Tauri 后端与前端产物。
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo 'src-tauri/Cargo.toml'

if (-not (Test-Path -LiteralPath $manifest)) {
    Write-Host '[SKIP]  无 src-tauri/Cargo.toml，构建阶段尚未开始'
    exit 0
}

& cargo build --manifest-path $manifest --all
if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

& deno task build:web
exit $LASTEXITCODE
