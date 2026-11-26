#requires -Version 7.4
<#
.SYNOPSIS
    T-16 录制交接回归入口。

.DESCRIPTION
    -Mode handoff：跑本 crate 的假清单 + 假解析自检（--self-test），结果不代表真实录制已验证。
    -Mode live（默认）：真实采集器尚未实现，刻意退出 1，避免假路径把 T-11 判据凑绿。
#>
[CmdletBinding()]
param(
    [ValidateSet('handoff', 'live')]
    [string]$Mode = 'live'
)

$ErrorActionPreference = 'Stop'
$repo = Split-Path -Parent $PSScriptRoot
$manifest = Join-Path $repo 'Cargo.toml'

switch ($Mode) {
    'handoff' {
        Write-Host '[FAKE] 假录像清单 + 假解析结果驱动；未接真实采集（Q-01 未决）；本结果不代表真实录制已验证'
        & cargo run --manifest-path $manifest -p tacticdeck-record -- --self-test
        exit $LASTEXITCODE
    }
    'live' {
        Write-Host '真实采集器尚未实现：Q-01（本地 HLAE 抓屏 vs 服务端录制）未决，record-round 无可执行实现'
        exit 1
    }
}
