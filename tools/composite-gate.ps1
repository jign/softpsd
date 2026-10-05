param(
    [Parameter(Mandatory = $true, Position = 0)][string]$Psd,
    [string]$Krita = '',
    [string]$Gimp = ''
)

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$inputPath = (Resolve-Path -LiteralPath $Psd).Path
$outputDirectory = Join-Path $root 'target\composites'
$null = New-Item -ItemType Directory -Force -Path $outputDirectory
$python = Join-Path $PSScriptRoot '.venv\Scripts\python.exe'
$programFolders = @($env:ProgramFiles, $env:ProgramW6432)
$programFolders += [System.IO.DriveInfo]::GetDrives() |
    Where-Object { $_.IsReady -and $_.DriveType -eq 'Fixed' } |
    ForEach-Object { Join-Path $_.RootDirectory.FullName 'Program Files' }
$programFolders = $programFolders | Where-Object { $_ } | Select-Object -Unique

function Find-Engine {
    param([string]$Override, [string[]]$Commands, [string]$RelativePath)
    if ($Override) { return (Resolve-Path -LiteralPath $Override).Path }
    foreach ($command in $Commands) {
        $found = Get-Command $command -CommandType Application -ErrorAction SilentlyContinue | Select-Object -First 1
        if ($found) { return $found.Source }
    }
    foreach ($folder in $programFolders) {
        $path = Join-Path $folder $RelativePath
        if (Test-Path -LiteralPath $path -PathType Leaf) { return $path }
    }
    return $null
}

function Compare-Export {
    param([string]$Engine, [string]$Png)
    if (-not (Test-Path -LiteralPath $Png -PathType Leaf)) { throw "$Engine produced no PNG" }
    & $python (Join-Path $PSScriptRoot 'readers\compare-merged.py') $inputPath $Png
    if ($LASTEXITCODE -eq 0) {
        Write-Output "composite ${Engine}: match"
    } elseif ($LASTEXITCODE -eq 1) {
        Write-Output "composite ${Engine}: differs (informational)"
    } else {
        throw "$Engine comparison failed: exit $LASTEXITCODE"
    }
}

try {
    $kritaExe = Find-Engine $Krita @('krita.exe') 'Krita (x64)\bin\krita.exe'
    $gimpExe = Find-Engine $Gimp @('gimp-console-3.exe', 'gimp-console-3.2.exe', 'gimp-console-3.0.exe') 'GIMP 3\bin\gimp-console-3.exe'
    $baseName = [System.IO.Path]::GetFileName($inputPath)
    if ($kritaExe) {
        Write-Output "=== Krita: $kritaExe ==="
        $png = Join-Path $outputDirectory "$baseName.krita.png"
        if (Test-Path -LiteralPath $png) { Remove-Item -LiteralPath $png }
        $arguments = @('--nosplash', '--export', ('"' + $inputPath + '"'), '--export-filename', ('"' + $png + '"'))
        $process = Start-Process -FilePath $kritaExe -ArgumentList $arguments -WindowStyle Hidden -PassThru
        $null = $process.Handle
        if (-not $process.WaitForExit(120000)) {
            Stop-Process -Id $process.Id
            throw 'Krita export timed out'
        }
        if ($process.ExitCode -ne 0) { throw "Krita export failed: exit $($process.ExitCode)" }
        Compare-Export 'Krita' $png
    } else {
        Write-Output 'skipped: Krita not installed'
    }
    if ($gimpExe) {
        Write-Output "=== GIMP: $gimpExe ==="
        $png = Join-Path $outputDirectory "$baseName.gimp.png"
        if (Test-Path -LiteralPath $png) { Remove-Item -LiteralPath $png }
        $source = $inputPath.Replace('\', '/').Replace('"', '\"')
        $destination = $png.Replace('\', '/').Replace('"', '\"')
        $script = @"
(begin
  (script-fu-use-v3)
  (gimp-context-push)
  (gimp-context-set-background "white")
  (let* ((image (gimp-file-load RUN-NONINTERACTIVE "$source")))
    (gimp-image-flatten image)
    (file-png-export #:run-mode RUN-NONINTERACTIVE #:image image #:file "$destination" #:options -1 #:include-color-profile #t)
    (gimp-image-delete image))
  (gimp-context-pop))
"@
        $log = Join-Path $outputDirectory "$baseName.gimp.log"
        $errorLog = Join-Path $outputDirectory "$baseName.gimp.stderr.log"
        $arguments = @('--new-instance', '--no-data', '--no-fonts', '--batch-interpreter=plug-in-script-fu-eval',
            '-b', ('"' + $script.Replace('"', '\"') + '"'), '--quit')
        $process = Start-Process -FilePath $gimpExe -ArgumentList $arguments -WindowStyle Hidden -PassThru -RedirectStandardOutput $log -RedirectStandardError $errorLog
        $null = $process.Handle
        if (-not $process.WaitForExit(120000)) {
            Stop-Process -Id $process.Id
            throw 'GIMP export timed out'
        }
        if ($process.ExitCode -ne 0) { throw "GIMP export failed: exit $($process.ExitCode); see $errorLog" }
        if (-not (Test-Path -LiteralPath $png)) {
            Get-Content -LiteralPath $errorLog -Tail 10
            throw "GIMP produced no PNG; see $log and $errorLog"
        }
        Compare-Export 'GIMP' $png
    } else {
        Write-Output 'skipped: GIMP not installed'
    }
    exit 0
} catch {
    Write-Error "Composite gate failed for '$inputPath': $_" -ErrorAction Continue
    exit 1
}
