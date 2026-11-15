# Starts `hg-launch-tool hold 150`; after $Delay s (0 = never) runs
# `hg-launch-tool round-trip` once; samples the hold's two children every
# 250 ms for $Secs s and prints each thread born after the first 2 s
# (wall time) and the round trip's start and end.
param([int]$Delay, [int]$Secs = 40)
$tool = Join-Path $PSScriptRoot 'target\debug\hg-launch-tool.exe'
$psi = New-Object System.Diagnostics.ProcessStartInfo $tool, 'hold 150'
$psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true
$h = [System.Diagnostics.Process]::Start($psi)
$pids = @()
while ($true) {
    $l = $h.StandardOutput.ReadLine()
    if ($l -eq $null) { break }
    if ($l.StartsWith('hold: children ')) {
        $pids = $l.Substring(15).Split(' ') | ForEach-Object { [int]$_.Split('=')[1] }; break
    }
}
$t0 = Get-Date; $seen = @{}; $rt = $null; $rtEnd = $null
while (((Get-Date) - $t0).TotalSeconds -lt $Secs) {
    if ($Delay -gt 0 -and -not $rt -and ((Get-Date) - $t0).TotalSeconds -ge $Delay) {
        $rt = Start-Process -PassThru -WindowStyle Hidden -FilePath $tool -ArgumentList 'round-trip'
        "round-trip start $((Get-Date).ToString('HH:mm:ss.fff'))"
    }
    if ($rt -and -not $rtEnd -and $rt.HasExited) { $rtEnd = Get-Date; "round-trip end   $($rtEnd.ToString('HH:mm:ss.fff')) exit=$($rt.ExitCode)" }
    foreach ($c in Get-Process -Id $pids -ErrorAction SilentlyContinue) {
        foreach ($t in $c.Threads) {
            $k = "$($c.Id)/$($t.Id)"
            if (-not $seen.ContainsKey($k)) {
                $seen[$k] = 1
                $age = ($t.StartTime - $c.StartTime).TotalSeconds
                if ($age -gt 2) { "  $($c.ProcessName) thread $($t.Id) born $($t.StartTime.ToString('HH:mm:ss.fff')) (+{0:N1}s)" -f $age }
            }
        }
    }
    Start-Sleep -Milliseconds 250
}
$n = Get-Process -Id $pids -ErrorAction SilentlyContinue | ForEach-Object { "$($_.ProcessName)=$($_.Threads.Count)" }
"threads at end: $($n -join ' ')"
$h.Kill(); $h.WaitForExit()
