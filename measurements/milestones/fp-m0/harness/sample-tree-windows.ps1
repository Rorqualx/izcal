# FP-M0 footprint sampler, Windows flavour (commit charge + working set).
# Usage: powershell -File sample-tree-windows.ps1 -Pattern <profile-dir> -Minutes 2 -Label <label>
# Sums PrivateMemorySize64 (the commit charge) and WorkingSet64 across the
# process tree rooted at CommandLine matches, one row every 5 s into
# results\<date>-<label>.tsv. Both counters are recorded so the
# counter-mapping observation (FP open question 17) can say which reported
# figure each incumbent's footprint follows.
param([string]$Pattern, [int]$Minutes = 2, [string]$Label = "manual")
$ErrorActionPreference = "SilentlyContinue"
$dir = Join-Path $PSScriptRoot "results"
New-Item -ItemType Directory -Force -Path $dir | Out-Null
$out = Join-Path $dir ((Get-Date -Format "yyyy-MM-dd") + "-" + $Label + ".tsv")

function Get-TreePids {
  # BFS walk over Win32_Process parentage, deduped; self excluded.
  $all = Get-CimInstance Win32_Process
  $roots = @($all | Where-Object { $_.CommandLine -match [regex]::Escape($Pattern) } |
             ForEach-Object { [int]$_.ProcessId } |
             Where-Object { $_ -ne $PID })
  $seen = @{}; $queue = New-Object System.Collections.Queue
  foreach ($r in $roots) { if (-not $seen.ContainsKey($r)) { $seen[$r] = $true; $queue.Enqueue($r) } }
  while ($queue.Count -gt 0) {
    $p = $queue.Dequeue()
    foreach ($c in $all) {
      if ($c.ParentProcessId -eq $p -and -not $seen.ContainsKey([int]$c.ProcessId)) {
        $seen[[int]$c.ProcessId] = $true
        $queue.Enqueue([int]$c.ProcessId)
      }
    }
  }
  return $seen.Keys
}

"t`tsample`tnprocs`ttotal_private_MB`ttotal_ws_MB`tmax_single_private_MB" | Set-Content -Encoding utf8 $out
$n = $Minutes * 12
for ($i = 0; $i -le $n; $i++) {
  $pids = @(Get-TreePids)
  $totalPriv = 0; $totalWs = 0; $max = 0; $count = 0
  foreach ($id in $pids) {
    $p = Get-Process -Id $id
    if (-not $p) { continue }
    $mb = [math]::Round($p.PrivateMemorySize64 / 1MB)
    $wsmb = [math]::Round($p.WorkingSet64 / 1MB)
    $count++; $totalPriv += $mb; $totalWs += $wsmb
    if ($mb -gt $max) { $max = $mb }
  }
  "$(Get-Date -Format HH:mm:ss)`t$i`t$count`t$totalPriv`t$totalWs`t$max" | Add-Content -Encoding utf8 $out
  Start-Sleep 5
}
Write-Output "done: $n samples, $count processes in the last sample -> $out"
