# FP-M0 Windows matrix, Firefox leg v3. Loads open as ONE invocation with
# N URLs (one window, N tabs). The v2 attach pattern (repeated
# invocations) polluted the process set: each -headless invocation against
# the locked profile lingered instead of attaching (296 procs at the
# cross-site load, ~4 MB stubs each, ~a third of the private total).
# Fresh instance per load, same profile dir, kill between loads.
# Evidence tier: process/memory growth, the macOS Firefox caveat.
param()
$ErrorActionPreference = "Continue"
$root = $PSScriptRoot
$ff = Join-Path $env:LOCALAPPDATA "Mozilla Firefox\firefox.exe"
if (-not (Test-Path $ff)) { $ff = "C:\Program Files\Mozilla Firefox\firefox.exe" }
if (-not (Test-Path $ff)) { Write-Output "firefox half skipped: firefox.exe not installed"; exit 0 }
$url = "https://example.com"
$fprof = Join-Path $root ("profiles\ff-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
New-Item -ItemType Directory -Force -Path $fprof | Out-Null
$sites = "example.com","example.org","iana.org","wikipedia.org","archive.org","openstreetmap.org","gutenberg.org","ietf.org","w3.org","python.org"

function Stop-Ff {
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match [regex]::Escape($fprof) } |
    ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep 2
}
function Run-Load($label, $urls) {
  Stop-Ff
  $arglist = @("-headless", "-profile", "$fprof") + $urls
  Start-Process -FilePath $ff -ArgumentList $arglist | Out-Null
  Start-Sleep 15
  Write-Output "firefox-$label"
  powershell -NoProfile -File "$root\sample-tree-windows.ps1" -Pattern $fprof -Minutes 2 -Label "firefox-${label}-v3"
}

Run-Load "blank" @("about:blank")
foreach ($t in 1, 10, 30) {
  $urls = @(); for ($i = 0; $i -lt $t; $i++) { $urls += $url }
  Run-Load "${t}tabs" $urls
}
$urls = @(); foreach ($s in $sites) { for ($j = 0; $j -lt 3; $j++) { $urls += "https://$s" } }
Run-Load "crosssite" $urls

Stop-Ff
Remove-Item -Recurse -Force $fprof -ErrorAction SilentlyContinue
Write-Output "firefox v3 done"
