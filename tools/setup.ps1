# Installs the external readers under tools/. Idempotent.
$ErrorActionPreference = "Stop"
$here = Split-Path -Parent $MyInvocation.MyCommand.Path
python -m venv "$here\.venv"
& "$here\.venv\Scripts\python.exe" -m pip install --quiet --disable-pip-version-check --upgrade psd-tools PhotoshopAPI
Push-Location "$here\readers"
npm install --silent
Pop-Location
"readers installed"
