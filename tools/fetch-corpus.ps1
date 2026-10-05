param()

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot

function Fetch-Corpus {
    param([string]$Name, [string]$Url, [string]$Subdirectory)
    $path = Join-Path $root "corpus\$Name"
    if (Test-Path -LiteralPath $path) {
        & git -C $path pull --ff-only
        if ($LASTEXITCODE -ne 0) { throw "Pull failed: $Name" }
    } else {
        & git clone --depth 1 --filter=blob:none --sparse $Url $path
        if ($LASTEXITCODE -ne 0) { throw "Clone failed: $Name" }
    }
    & git -C $path sparse-checkout set $Subdirectory
    if ($LASTEXITCODE -ne 0) { throw "Sparse checkout failed: $Name" }
    $files = @(Get-ChildItem -LiteralPath (Join-Path $path $Subdirectory) -Recurse -File |
        Where-Object { $_.Extension -in '.psd', '.psb' })
    $psd = @($files | Where-Object Extension -EQ '.psd').Count
    $psb = @($files | Where-Object Extension -EQ '.psb').Count
    Write-Output "${Name}: $psd PSD, $psb PSB files"
}

Fetch-Corpus 'ag-psd' 'https://github.com/Agamnentzar/ag-psd.git' 'test'
Fetch-Corpus 'psd-tools' 'https://github.com/psd-tools/psd-tools.git' 'tests/psd_files'
