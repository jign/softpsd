param(
    [Parameter(Mandatory = $true, Position = 0)]
    [string]$Psd
)

$ErrorActionPreference = 'Stop'
$psdPath = (Resolve-Path -LiteralPath $Psd).Path
$root = Split-Path -Parent $PSScriptRoot
$python = Join-Path $PSScriptRoot '.venv\Scripts\python.exe'

Push-Location $root
try {
    $dump = @(& cargo run --example dump -- $psdPath)
    if ($LASTEXITCODE -ne 0) {
        throw 'softpsd dump failed'
    }
    $readerTree = @($dump | Where-Object { $_ -match '^\s*(group|layer) ' })
    Write-Output '=== softpsd ==='
    $readerTree | Write-Output

    $photoshop = & (Join-Path $PSScriptRoot 'photoshop\run.ps1') (Join-Path $PSScriptRoot 'photoshop\dump.jsx') -Out $psdPath
    $photoshopTree = @($photoshop -split '\r?\n' | ForEach-Object {
        if ($_ -match '^\s*group ') {
            $_ -replace '( visible=(?:true|false)) bounds=[^ ]+', '$1'
        } else {
            $_
        }
    } | Where-Object { $_ -ne '' })
    Write-Output '=== Photoshop ==='
    $photoshopTree | Write-Output

    $lineCount = [Math]::Max($readerTree.Count, $photoshopTree.Count)
    for ($index = 0; $index -lt $lineCount; $index++) {
        $readerLine = if ($index -lt $readerTree.Count) { $readerTree[$index] } else { '<missing>' }
        $photoshopLine = if ($index -lt $photoshopTree.Count) { $photoshopTree[$index] } else { '<missing>' }
        if ($readerLine -cne $photoshopLine) {
            throw "Tree differs at line $($index + 1): softpsd=$readerLine; Photoshop=$photoshopLine"
        }
    }

    Write-Output '=== layer comparison ==='
    & $python (Join-Path $PSScriptRoot 'readers\compare-layers.py') $psdPath
    if ($LASTEXITCODE -ne 0) {
        throw 'Layer comparison failed'
    }
} catch {
    Write-Error $_ -ErrorAction Continue
    exit 1
} finally {
    Pop-Location
}
