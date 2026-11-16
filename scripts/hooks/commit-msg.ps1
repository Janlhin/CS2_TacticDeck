#requires -Version 7.4
<#
.SYNOPSIS
    commit-msg 钩子逻辑：校验提交信息符合 Conventional Commits。
.PARAMETER MsgFile
    Git 传入的提交信息文件路径（由薄壳以 "$1" 转发）。
#>
param(
    [Parameter(Mandatory = $true)]
    [string]$MsgFile
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$bin = Join-Path $repo 'node_modules/.bin/commitlint.cmd'
$cfg = Join-Path $repo 'config/commitlint.config.mjs'

if (-not (Test-Path -LiteralPath $bin)) {
    Write-Host '[FAIL]   未安装 commitlint（先运行 npm ci）'
    exit 1
}

& $bin --config $cfg --edit $MsgFile
exit $LASTEXITCODE
