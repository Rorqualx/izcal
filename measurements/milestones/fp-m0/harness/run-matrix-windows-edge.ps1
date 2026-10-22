# FP-M0 Windows matrix, Edge half (the Windows Chromium incumbent; no
# Chrome on this rig). Fresh profile, blank/1/10/30 same-site tabs plus a
# 30-tab cross-site mix, 2-minute samples. Tabs go through Edge's DevTools
# /json/new endpoint with verified page counts.
param()
$ErrorActionPreference = "Continue"
$root = $PSScriptRoot
$edge = "C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe"
$url = "https://example.com"
$cprof = Join-Path $root ("profiles\edge-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
New-Item -ItemType Directory -Force -Path $cprof | Out-Null
$cport = 9345
$sites = "example.com","example.org","iana.org","wikipedia.org","archive.org","openstreetmap.org","gutenberg.org","ietf.org","w3.org","python.org"

Start-Process -FilePath $edge -ArgumentList "--headless=new","--user-data-dir=$cprof","--remote-debugging-port=$cport","about:blank" | Out-Null
Start-Sleep 12
function Get-Pages {
  try {
    $d = Invoke-RestMethod -Uri "http://127.0.0.1:$cport/json" -TimeoutSec 5
    return @($d | Where-Object { $_.type -eq "page" }).Count
  } catch { return 0 }
}
function Add-Tabs($want) {
  for ($i = 0; $i -lt $want; $i++) {
    Invoke-RestMethod -Method Put -Uri "http://127.0.0.1:$cport/json/new?$url" -TimeoutSec 5 | Out-Null
    Start-Sleep -Milliseconds 400
  }
  Start-Sleep 6
  Write-Output "  edge: $(Get-Pages) pages (want >= $want)"
}
Write-Output "edge-blank"
powershell -NoProfile -File "$root\sample-tree-windows.ps1" -Pattern $cprof -Minutes 2 -Label "edge-blank-v1"
foreach ($t in 1, 10, 30) {
  Add-Tabs $t
  powershell -NoProfile -File "$root\sample-tree-windows.ps1" -Pattern $cprof -Minutes 2 -Label "edge-${t}tabs-v1"
}
foreach ($s in $sites) {
  for ($j = 0; $j -lt 3; $j++) {
    Invoke-RestMethod -Method Put -Uri "http://127.0.0.1:$cport/json/new?https://$s" -TimeoutSec 5 | Out-Null
    Start-Sleep -Milliseconds 400
  }
}
Start-Sleep 8
Write-Output "edge-cross: $(Get-Pages) pages"
powershell -NoProfile -File "$root\sample-tree-windows.ps1" -Pattern $cprof -Minutes 2 -Label "edge-crosssite-v1"
Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -match [regex]::Escape($cprof) } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Start-Sleep 2
Write-Output "edge half done"
