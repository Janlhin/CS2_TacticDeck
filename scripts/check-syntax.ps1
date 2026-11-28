#requires -Version 7.4
<#
.SYNOPSIS
    PowerShell 脚本纯语法检查：用 Parser 解析全部 .ps1，不执行脚本本身。

.DESCRIPTION
    不依赖 PSScriptAnalyzer 模块。只做语法解析，报出解析错误；
    扫描范围为 scripts/ 目录及其子目录下的 .ps1 文件。

.EXAMPLE
    pwsh -NoLogo -NoProfile -File ./scripts/check-syntax.ps1
#>
[CmdletBinding()]
param(
    [string]$RepoRoot = (Split-Path -Parent $PSScriptRoot)
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$problems = [System.Collections.Generic.List[string]]::new()

$ps1Files = @(
    Get-ChildItem -LiteralPath (Join-Path $RepoRoot 'scripts') -Recurse -Filter '*.ps1' -File
)

foreach ($file in $ps1Files) {
    $tokens = $null
    $errors = $null
    [System.Management.Automation.Language.Parser]::ParseFile(
        $file.FullName, [ref]$tokens, [ref]$errors
    ) | Out-Null
    if ($errors -and $errors.Count -gt 0) {
        $relative = $file.FullName.Substring($RepoRoot.Length + 1)
        foreach ($err in $errors) {
            $problems.Add("$relative : $($err.Message)")
        }
    }
}

if ($problems.Count -gt 0) {
    Write-Host "PowerShell 语法检查未通过，共 $($problems.Count) 项：" -ForegroundColor Red
    foreach ($problem in $problems) { Write-Host "  - $problem" -ForegroundColor Red }
    exit 1
}

Write-Host "PowerShell 语法检查通过：$($ps1Files.Count) 个 .ps1 无解析错误。" -ForegroundColor Green
exit 0
