param()

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$report = Join-Path $root 'target\corpus-triage.txt'
$lines = Get-Content -LiteralPath (Join-Path $root 'target\corpus.txt') -Encoding UTF8
$output = [System.IO.StreamWriter]::new($report, $false, [System.Text.UTF8Encoding]::new($false))
try {
    foreach ($line in $lines) {
        $fields = $line.Split("`t", 2)
        if (-not $fields[0].StartsWith('malformed: ')) { continue }
        $path = Join-Path $root $fields[1]
        try {
            $result = & (Join-Path $PSScriptRoot 'photoshop\run.ps1') (Join-Path $PSScriptRoot 'photoshop\opens.jsx') -Out $path
        } catch {
            $result = "error: $_"
        }
        $result = ([string]$result).Replace("`r", ' ').Replace("`n", ' ').Replace("`t", ' ')
        $output.WriteLine("$result`t$($fields[1])`t$($fields[0])")
        $output.Flush()
        Write-Output "$result`t$($fields[1])"
    }
} finally {
    $output.Dispose()
}
