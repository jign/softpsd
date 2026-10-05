param(
    [Parameter(Position = 0, ValueFromRemainingArguments = $true)]
    [string[]]$Names = @(),
    [switch]$Make
)

$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$fixtureName = '<catalog>'
$stage = 'list'

function Invoke-FixtureGate {
    param([scriptblock]$Command)
    Write-Output "=== fixture ${fixtureName}: $stage ==="
    & $Command
    if ($LASTEXITCODE -ne 0) {
        throw "exit $LASTEXITCODE"
    }
}

Push-Location $root
try {
    $knownNames = @(& cargo run --quiet --example fixture -- --list)
    if ($LASTEXITCODE -ne 0) {
        throw "exit $LASTEXITCODE"
    }
    if ($Names.Count -eq 0) {
        $Names = $knownNames
    }
    foreach ($name in $Names) {
        $fixtureName = $name
        $stage = 'select'
        if ($knownNames -cnotcontains $name) {
            throw 'unknown fixture'
        }
    }
    foreach ($name in $Names) {
        $fixtureName = $name
        $ours = Join-Path $root "target\softpsd-$name.psd"
        $photoshop = Join-Path $root "tests\fixtures\ps27-$name.psd"
        if ($Make) {
            $stage = 'make Photoshop fixture'
            Invoke-FixtureGate {
                & (Join-Path $PSScriptRoot 'photoshop\run.ps1') (Join-Path $PSScriptRoot "photoshop\fixtures\$name.jsx") -Out $photoshop
            }
        }
        $stage = 'build'
        Invoke-FixtureGate { & cargo run --example fixture -- $name $ours }
        $stage = 'writer gate'
        Invoke-FixtureGate { & (Join-Path $PSScriptRoot 'writer-gate.ps1') $ours }
        $stage = 'reader gate (softpsd)'
        Invoke-FixtureGate { & (Join-Path $PSScriptRoot 'reader-gate.ps1') $ours }
        if (Test-Path -LiteralPath $photoshop) {
            $stage = 'reader gate (Photoshop)'
            Invoke-FixtureGate { & (Join-Path $PSScriptRoot 'reader-gate.ps1') $photoshop }
        }
        Write-Output "fixture ${fixtureName}: passed"
    }
} catch {
    Write-Error "Fixture '$fixtureName' failed at '$stage': $_" -ErrorAction Continue
    exit 1
} finally {
    Pop-Location
}
