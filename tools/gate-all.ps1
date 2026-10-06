param()

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$step = 'setup check'

function Invoke-Check {
    param([scriptblock]$Command)
    Write-Output "=== $step ==="
    & $Command
    if ($LASTEXITCODE -ne 0) { throw "exit $LASTEXITCODE" }
}

Push-Location $root
try {
    if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot '.venv\Scripts\python.exe')) -or
        -not (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'readers\node_modules'))) {
        throw 'External readers missing. Run tools\setup.ps1 first.'
    }
    if (-not (Test-Path -LiteralPath (Join-Path $PSScriptRoot 'wasmtime\wasmtime.exe'))) {
        throw 'wasmtime missing. Run tools\setup.ps1 first.'
    }
    Write-Output 'setup: external reader directories present'
    $step = 'cargo fmt --check'
    Invoke-Check { & cargo fmt --check }
    $step = 'cargo clippy --all-targets'
    Invoke-Check { & cargo clippy --all-targets }
    $step = 'cargo test'
    Invoke-Check { & cargo test }
    $step = 'cargo doc'
    $env:RUSTDOCFLAGS = '-D warnings'
    Invoke-Check { & cargo doc --no-deps --quiet }
    Remove-Item Env:RUSTDOCFLAGS
    $step = 'cargo package'
    Invoke-Check { & cargo package --allow-dirty --quiet }
    $step = 'fuzz'
    Invoke-Check { & cargo run --release --quiet --example fuzz -- 7 200000 }
    $step = 'wasm32-unknown-unknown build'
    Invoke-Check { & cargo build --quiet --lib --target wasm32-unknown-unknown }
    $step = 'wasm32-wasip1 tests'
    Invoke-Check { & cargo test --quiet --target wasm32-wasip1 }
    $step = 'wasm32-wasip1 fuzz'
    Invoke-Check { & cargo run --release --quiet --target wasm32-wasip1 --example fuzz -- 7 100000 }
    $step = 'fixture gates'
    Invoke-Check { & (Join-Path $PSScriptRoot 'gate-fixtures.ps1') }
    if (Test-Path -LiteralPath (Join-Path $root 'corpus')) {
        $step = 'cargo test --features corpus'
        Invoke-Check { & cargo test --features corpus }
    } else {
        Write-Output 'skipped: corpus not fetched'
    }
    $fixtures = Get-ChildItem -LiteralPath (Join-Path $root 'tests\fixtures') -File |
        Where-Object { $_.Name -like 'softpsd-*' -and $_.Extension -in '.psd', '.psb' } |
        Sort-Object Name
    foreach ($fixture in $fixtures) {
        $step = "composites: $($fixture.Name)"
        Invoke-Check { & (Join-Path $PSScriptRoot 'composite-gate.ps1') $fixture.FullName }
    }
    Write-Output 'gate-all: passed'
} catch {
    Write-Error "Gate failed at '$step': $_" -ErrorAction Continue
    exit 1
} finally {
    Pop-Location
}
