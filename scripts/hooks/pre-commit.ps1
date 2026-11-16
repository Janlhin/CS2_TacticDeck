#requires -Version 7.4
<#
.SYNOPSIS
    pre-commit 钩子逻辑：运行快速验收（不查外链、不实跑文档命令）。
.DESCRIPTION
    钩子薄壳（.husky/pre-commit）只负责 exec pwsh 转发到本文件；业务逻辑只存在于 .ps1。
#>
$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)

& pwsh -NoLogo -NoProfile -File (Join-Path $repo 'scripts/verify.ps1') -NoExternal -NoDocCommands
exit $LASTEXITCODE
