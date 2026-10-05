# Snapshots external references into doc/ref/local/ (gitignored).
# adobe.com times out from some networks; the Wayback copy is used.
$ErrorActionPreference = "Stop"
$local = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "local"
New-Item -ItemType Directory -Force $local | Out-Null
$items = @(
    @("adobe-psd-spec.html",  "https://web.archive.org/web/2025/https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/"),
    @("ag-psd-README_PSD.md", "https://raw.githubusercontent.com/Agamnentzar/ag-psd/master/README_PSD.md"),
    @("gimp-psd-load.c",      "https://gitlab.gnome.org/GNOME/gimp/-/raw/master/plug-ins/file-psd/psd-load.c"),
    @("gimp-psd-export.c",    "https://gitlab.gnome.org/GNOME/gimp/-/raw/master/plug-ins/file-psd/psd-export.c"),
    @("gimp-psd-layer-res-load.c", "https://gitlab.gnome.org/GNOME/gimp/-/raw/master/plug-ins/file-psd/psd-layer-res-load.c")
)
foreach ($it in $items) {
    $dest = Join-Path $local $it[0]
    & curl.exe -sS -L -m 60 -A "Mozilla/5.0" -o $dest $it[1]
    if ($LASTEXITCODE -ne 0) { throw "fetch failed: $($it[1])" }
    if ($it[0] -notlike "*.html" -and (Get-Content $dest -TotalCount 1) -like "<!DOCTYPE html>*") { throw "got an HTML page, not the file: $($it[1])" }
    "fetched $($it[0]) ($((Get-Item $dest).Length) bytes)"
}
