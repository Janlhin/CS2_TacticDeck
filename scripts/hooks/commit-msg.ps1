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
$bin = Join-Path $repo 'node_modules/@commitlint/cli/cli.js'
$cfg = Join-Path $repo 'config/commitlint.config.mjs'

if (-not (Test-Path -LiteralPath $bin)) {
    Write-Host '[FAIL]   未安装 commitlint（先运行 deno install）'
    exit 1
}

& deno run -A $bin --config $cfg --edit $MsgFile
exit $LASTEXITCODE
