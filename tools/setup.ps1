# Installs the external readers under tools/. Idempotent.
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
python -m venv "$here\.venv"
& "$here\.venv\Scripts\python.exe" -m pip install --quiet --disable-pip-version-check --upgrade psd-tools PhotoshopAPI
Push-Location "$here\readers"
npm install --silent
Pop-Location
"readers installed"

rustup target add wasm32-unknown-unknown wasm32-wasip1
if ($LASTEXITCODE -ne 0) { throw "rustup target add failed" }
$wasmtimeVersion = "v49.0.2"
$wasmtimeDir = "$here\wasmtime"
$wasmtimeExe = "$wasmtimeDir\wasmtime.exe"
$installed = if (Test-Path $wasmtimeExe) { & $wasmtimeExe --version } else { "" }
if ($installed -notlike "*$($wasmtimeVersion.TrimStart('v'))*") {
    $name = "wasmtime-$wasmtimeVersion-x86_64-windows"
    $zip = "$env:TEMP\$name.zip"
    Invoke-WebRequest "https://github.com/bytecodealliance/wasmtime/releases/download/$wasmtimeVersion/$name.zip" -OutFile $zip
    Expand-Archive $zip -DestinationPath $env:TEMP -Force
    New-Item -ItemType Directory -Force $wasmtimeDir | Out-Null
    Copy-Item "$env:TEMP\$name\wasmtime.exe" $wasmtimeExe -Force
    Remove-Item $zip
}
"wasmtime $(& $wasmtimeExe --version)"
