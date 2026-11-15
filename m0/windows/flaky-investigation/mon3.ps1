# Idle-thread flake monitor (unelevated). Keeps $Holds copies of
# `hg-launch-tool hold 600` running (each holds two AppContainer children),
# plus two controls outside any sandbox (substrate-idle.exe, PING), for
# $Minutes. Samples every tracked process every 250 ms and logs to $Out each
# thread born when its process is older than 3 s (wall time, start address
# as module+offset) and each such thread's exit (lifetime).
param([int]$Holds = 4, [int]$Minutes = 75, [string]$Out = 'mon.log', [int]$HoldSecs = 600, [switch]$RoundTrip)
$rt = $null
$counts = @{}   # pid -> last thread count
$dir = Join-Path $env:USERPROFILE 'izcal-flaky\target\debug'
$tool = Join-Path $dir 'hg-launch-tool.exe'
$out = Join-Path $PSScriptRoot $Out
Set-Content $out "start $((Get-Date).ToString('yyyy-MM-dd HH:mm:ss.fff')) holds=$Holds minutes=$Minutes"
function Log($s) { Add-Content $out $s }
$tracked = @{}   # pid -> label
$slots = @{}     # slot -> @{ broker; file; parsed }
function Start-Hold($slot) {
    $f = Join-Path $env:TEMP "mon-hold-$slot.txt"
    $b = Start-Process -PassThru -WindowStyle Hidden -FilePath $tool -ArgumentList 'hold', "$HoldSecs" -RedirectStandardOutput $f
    $slots[$slot] = @{ broker = $b; file = $f; parsed = $false }
    $tracked[$b.Id] = "broker$slot"
    Log "$((Get-Date).ToString('HH:mm:ss.fff')) hold $slot started broker=$($b.Id)"
}
for ($s = 0; $s -lt $Holds; $s++) { Start-Hold $s }
$ctl = @(
    (Start-Process -PassThru -WindowStyle Hidden -FilePath (Join-Path $dir 'substrate-idle.exe')),
    (Start-Process -PassThru -WindowStyle Hidden -FilePath "$env:SystemRoot\System32\PING.EXE" -ArgumentList '-n', '100000', '127.0.0.1')
)
$tracked[$ctl[0].Id] = 'ctl-idle'; $tracked[$ctl[1].Id] = 'ctl-ping'
$seen = @{}      # "pid/tid" -> born DateTime (or $null for start-up threads)
$mods = @{}      # pid -> module list
function Where-Addr($p, $addr) {
    if (-not $mods.ContainsKey($p.Id)) { try { $mods[$p.Id] = @($p.Modules) } catch { $mods[$p.Id] = @() } }
    $a = [int64]$addr
    foreach ($m in $mods[$p.Id]) {
        $base = [int64]$m.BaseAddress
        if ($a -ge $base -and $a -lt $base + $m.ModuleMemorySize) { return '{0}+0x{1:x}' -f $m.ModuleName, ($a - $base) }
    }
    return '0x{0:x}' -f $a
}
$t0 = Get-Date
while (((Get-Date) - $t0).TotalMinutes -lt $Minutes) {
    foreach ($slot in @($slots.Keys)) {
        $h = $slots[$slot]
        if (-not $h.parsed) {
            $txt = Get-Content $h.file -ErrorAction SilentlyContinue | Where-Object { $_ -like 'hold: children *' }
            if ($txt) {
                foreach ($kv in $txt.Substring(15).Split(' ')) { $k, $v = $kv.Split('='); $tracked[[int]$v] = "$k$slot" }
                $h.parsed = $true
                Log "$((Get-Date).ToString('HH:mm:ss.fff')) hold $slot children $($txt.Substring(15))"
            }
        }
        if ($h.broker.HasExited) {
            Log "$((Get-Date).ToString('HH:mm:ss.fff')) hold $slot ended exit=$($h.broker.ExitCode)"
            Start-Hold $slot
        }
    }
    if ($RoundTrip) {
        if ($rt -and $rt.HasExited) { Log "$((Get-Date).ToString('HH:mm:ss.fff')) RT end exit=$($rt.ExitCode)"; $rt = $null }
        if (-not $rt) { $rt = Start-Process -PassThru -WindowStyle Hidden -FilePath $tool -ArgumentList 'round-trip'; Log "$((Get-Date).ToString('HH:mm:ss.fff')) RT start pid=$($rt.Id)" }
    }
    $now = Get-Date
    foreach ($procId in @($tracked.Keys)) {
        $p = Get-Process -Id $procId -ErrorAction SilentlyContinue
        if (-not $p) { $counts.Remove($procId); $tracked.Remove($procId); $mods.Remove($procId); continue }
        $st = $null; try { $st = $p.StartTime } catch {}
        if (-not $st) { continue }
        $live = @{}
        $n = $p.Threads.Count
        if ($tracked[$procId] -match '^(Net|Ren)' -and $counts[$procId] -ne $n) { $counts[$procId] = $n; Log ('{0} COUNT {1} pid={2} age={3:N2}s n={4}' -f $now.ToString('HH:mm:ss.fff'), $tracked[$procId], $procId, ($now - $st).TotalSeconds, $n) }
        foreach ($t in $p.Threads) {
            $k = "$procId/$($t.Id)"; $live[$k] = 1
            if (-not $seen.ContainsKey($k)) {
                $bt = $null; try { $bt = $t.StartTime } catch {}
                $age = if ($bt) { ($bt - $st).TotalSeconds } else { -1 }
                if ($age -gt 3 -or $tracked[$procId] -match '^(Net|Ren)') {
                    $seen[$k] = $bt
                    $addr = try { Where-Addr $p $t.StartAddress } catch { '?' }
                    Log ('{0} BORN {1} pid={2} tid={3} age={4:N1}s start={5}' -f $bt.ToString('HH:mm:ss.fff'), $tracked[$procId], $procId, $t.Id, $age, $addr)
                } else { $seen[$k] = $null }
            }
        }
        foreach ($k in @($seen.Keys)) {
            if ($k.StartsWith("$procId/") -and -not $live.ContainsKey($k)) {
                if ($seen[$k]) { Log ('{0} GONE {1} tid={2} lived~{3:N1}s' -f $now.ToString('HH:mm:ss.fff'), $tracked[$procId], $k.Split('/')[1], ($now - $seen[$k]).TotalSeconds) }
                $seen.Remove($k)
            }
        }
    }
    Start-Sleep -Milliseconds 250
}
foreach ($slot in @($slots.Keys)) { $b = $slots[$slot].broker; if (-not $b.HasExited) { $b.Kill() } }
$ctl | ForEach-Object { if (-not $_.HasExited) { $_.Kill() } }
Log "end $((Get-Date).ToString('HH:mm:ss.fff'))"
