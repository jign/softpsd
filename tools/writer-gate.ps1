param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Psd
)

$ErrorActionPreference = 'Stop'
$psdPath = (Resolve-Path -LiteralPath $Psd).Path
$python = Join-Path $PSScriptRoot '.venv\Scripts\python.exe'

function Invoke-GateStep {
    param([string]$Title, [string]$Command, [string[]]$Arguments)
    Write-Output "=== $Title ==="
    & $Command @Arguments
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

try {
    Invoke-GateStep 'psd-tools' $python @((Join-Path $PSScriptRoot 'readers\psd-tools-dump.py'), $psdPath)
    Invoke-GateStep 'ag-psd' 'node' @((Join-Path $PSScriptRoot 'readers\ag-psd-dump.js'), $psdPath)

    Write-Output '=== Photoshop ==='
    & (Join-Path $PSScriptRoot 'photoshop\run.ps1') (Join-Path $PSScriptRoot 'photoshop\dump.jsx') -Out $psdPath -ErrorAction Stop

    Invoke-GateStep 'merged comparison' $python @((Join-Path $PSScriptRoot 'readers\compare-merged.py'), $psdPath, "$psdPath.png")
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
}
