# FP-M0 idle-wakeup counter, Windows: reads the raw, cumulative
# "Context Switches" count of every thread of the process
# (Win32_PerfRawData_PerfProc_Thread, instances "<name>/<n>") at the
# start and end of a window. Each switch onto a thread is that thread
# being scheduled, so the difference per second approximates wakeups per
# second. Threads that start or exit inside the window are listed, since
# their counts are incomplete. CPU time from the process's
# TotalProcessorTime.
# Usage: powershell -File wakeups-windows.ps1 -Name fp-m0-empty -Seconds 60
param([string]$Name = "fp-m0-empty", [int]$Seconds = 60)
function Snap {
  $t = @{}
  Get-CimInstance Win32_PerfRawData_PerfProc_Thread -Filter "Name LIKE '$Name/%'" |
    ForEach-Object { $t["$($_.IDThread)"] = [int64]$_.ContextSwitchesPersec }
  $t
}
$p = Get-Process $Name | Select-Object -First 1
$cpu0 = $p.TotalProcessorTime.TotalMilliseconds
$a = Snap
Start-Sleep -Seconds $Seconds
$b = Snap
$p.Refresh(); $cpu1 = $p.TotalProcessorTime.TotalMilliseconds
$sw = 0; foreach ($k in $b.Keys) { if ($a.ContainsKey($k)) { $sw += $b[$k] - $a[$k] } }
$gone = @($a.Keys | Where-Object { -not $b.ContainsKey($_) }).Count
$new = @($b.Keys | Where-Object { -not $a.ContainsKey($_) }).Count
"pid={0} window={1}s threads={2} context_switches={3} ({4:N3}/s) threads_exited={5} threads_started={6} cpu={7:N1} ms" -f $p.Id, $Seconds, $b.Count, $sw, ($sw / $Seconds), $gone, $new, ($cpu1 - $cpu0)
