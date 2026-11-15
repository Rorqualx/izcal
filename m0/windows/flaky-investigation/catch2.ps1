# Runs the hg-launch-tool test binary on 4 CPUs; while it runs, samples the
# children (hg-net-main, hg-renderer-main) every second for 45 s and logs
# each thread's creation and exit (ms since the child started) and each
# module loaded after the first sample. Prints the log of a failing run
# only, then stops. Usage: powershell -NoProfile -File catch2.ps1 EXE MAXRUNS
param([string]$Exe, [int]$Max)
$exe = (Resolve-Path $Exe).Path
for ($i = 1; $i -le $Max; $i++) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo $exe, '--nocapture'
    $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    $ctl = @(
        (Start-Process -PassThru -WindowStyle Hidden -FilePath (Join-Path $PSScriptRoot 'target\debug\substrate-idle.exe')),
        (Start-Process -PassThru -WindowStyle Hidden -FilePath "$env:SystemRoot\System32\PING.EXE" -ArgumentList '-n','200','127.0.0.1')
    )
    $ctlIds = $ctl | ForEach-Object Id
    $p = [System.Diagnostics.Process]::Start($psi)
    $p.ProcessorAffinity = [IntPtr]15
    $o = $p.StandardOutput.ReadToEndAsync(); $e = $p.StandardError.ReadToEndAsync()
    $log = New-Object System.Collections.Generic.List[string]
    $seenT = @{}; $seenM = @{}; $seenS = @{}; $t0 = Get-Date
    while (((Get-Date) - $t0).TotalSeconds -lt 47 -and -not $p.HasExited) {
        foreach ($c in (@(Get-Process hg-net-main, hg-renderer-main -ErrorAction SilentlyContinue) + @(Get-Process -Id $ctlIds -ErrorAction SilentlyContinue))) {
            $st = $null; try { $st = $c.StartTime } catch {}
            if (-not $st) { continue }
            $key = "$($c.Id)"
            $now = Get-Date
            $ms = [int]($now - $st).TotalMilliseconds
            $live = @{}
            foreach ($t in $c.Threads) {
                $live["$key/$($t.Id)"] = 1
                if (-not $seenT.ContainsKey("$key/$($t.Id)")) {
                    $born = try { [int]($t.StartTime - $st).TotalMilliseconds } catch { -1 }
                    $seenT["$key/$($t.Id)"] = 1
                    $wall = try { $t.StartTime.ToString('HH:mm:ss.fff') } catch { '?' }
                    $log.Add(('{0} {1} +{2}ms: thread {3} born +{4}ms at {6} ({5})' -f $c.ProcessName, $c.Id, $ms, $t.Id, $born, $t.WaitReason, $wall))
                }
            }
            foreach ($k in @($seenT.Keys)) {
                if ($k.StartsWith("$key/") -and -not $live.ContainsKey($k) -and $seenT[$k] -eq 1) {
                    $seenT[$k] = 0
                    $log.Add(('{0} {1} +{2}ms: thread {3} gone' -f $c.ProcessName, $c.Id, $ms, $k.Split('/')[1]))
                }
            }
            try {
                foreach ($m in $c.Modules) {
                    $mk = "$key/$($m.ModuleName)"
                    if (-not $seenM.ContainsKey($mk)) {
                        $seenM[$mk] = 1
                        if ($seenS.ContainsKey($key)) { $log.Add(('{0} {1} +{2}ms: module {3}' -f $c.ProcessName, $c.Id, $ms, $m.ModuleName)) }
                    }
                }
                $seenS[$key] = 1
            } catch {}
        }
        Start-Sleep -Milliseconds 1000
    }
    $p.WaitForExit()
    $ctl | ForEach-Object { if (-not $_.HasExited) { $_.Kill() } }
    $line = ($e.Result + $o.Result) -split "`n" | Where-Object { $_ -match 'threads after|test result' }
    "run $i exit=$($p.ExitCode) $($line -join ' | ')"
    $log | Where-Object { $_ -match 'born \+(\d+)ms' -and [int]$Matches[1] -gt 2000 -or $_ -match 'module' } | ForEach-Object { "  $_" }
    if ($p.ExitCode -ne 0) { "  full log:"; $log | ForEach-Object { "  $_" }; break }
}
