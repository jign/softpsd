# Runs a .jsx in the open Photoshop and prints what it returns.
# -Out is exposed to the script as OUT (forward slashes). Env vars do not reach Photoshop.
param(
    [Parameter(Mandatory)][string]$Script,
    [string]$Out = ""
)
$js = [System.IO.File]::ReadAllText((Resolve-Path $Script))
if ($Out -ne "") {
    $abs = [System.IO.Path]::GetFullPath($Out).Replace("\", "/")
    $js = "var OUT = `"$abs`";`n" + $js
}
$ps = New-Object -ComObject Photoshop.Application
$ps.DoJavaScript($js)
