# Snapshots external references into doc/ref/local/ (gitignored).
# adobe.com times out from some networks; the Wayback copy is used.
$ErrorActionPreference = "Stop"
$local = Join-Path (Split-Path -Parent $MyInvocation.MyCommand.Path) "local"
New-Item -ItemType Directory -Force $local | Out-Null
$items = @(
    @("adobe-psd-spec.html",  "https://web.archive.org/web/2025/https://www.adobe.com/devnet-apps/photoshop/fileformatashtml/"),
    @("ag-psd-README_PSD.md", "https://raw.githubusercontent.com/Agamnentzar/ag-psd/master/README_PSD.md")
)
foreach ($it in $items) {
    $dest = Join-Path $local $it[0]
    & curl.exe -sS -L -m 60 -A "Mozilla/5.0" -o $dest $it[1]
    if ($LASTEXITCODE -ne 0) { throw "fetch failed: $($it[1])" }
    if ($it[0] -notlike "*.html" -and (Get-Content $dest -TotalCount 1) -like "<!DOCTYPE html>*") { throw "got an HTML page, not the file: $($it[1])" }
    "fetched $($it[0]) ($((Get-Item $dest).Length) bytes)"
}

# Reference clones. Shallow; the big repos are sparse to their PSD code only.
$repos = @(
    @("ag-psd",       "https://github.com/Agamnentzar/ag-psd.git",         @()),
    @("psd-tools",    "https://github.com/psd-tools/psd-tools.git",        @()),
    @("PhotoshopAPI", "https://github.com/EmilDohne/PhotoshopAPI.git",     @("PhotoshopAPI")),
    @("psd-rs",       "https://github.com/chinedufn/psd.git",              @()),
    @("psd_sdk",      "https://github.com/MolecularMatters/psd_sdk.git",   @()),
    @("gimp",         "https://gitlab.gnome.org/GNOME/gimp.git",           @("plug-ins/file-psd")),
    @("krita",        "https://invent.kde.org/graphics/krita.git",         @("libs/psd", "plugins/impex/psd"))
)
foreach ($r in $repos) {
    $dest = Join-Path $local $r[0]
    if (Test-Path $dest) { Push-Location $dest; git pull -q --depth 1; Pop-Location; "updated $($r[0])"; continue }
    if ($r[2].Count -eq 0) {
        git clone -q --depth 1 $r[1] $dest
    } else {
        git clone -q --depth 1 --filter=blob:none --sparse $r[1] $dest
        Push-Location $dest; git sparse-checkout set $r[2]; Pop-Location
    }
    "cloned $($r[0])"
}
