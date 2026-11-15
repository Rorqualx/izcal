# Runs `hg-launch-tool hold 150` as an_idle_child_runs_two_threads does,
# then samples both children's threads at the given seconds.
# Usage: powershell -NoProfile -File hold.ps1 [seconds...]
param([Parameter(ValueFromRemainingArguments = $true)][int[]]$At)
if (-not $At) { $At = @(1, 5, 15, 30, 45, 60, 90, 100) }
$tool = Join-Path $PSScriptRoot 'target\debug\hg-launch-tool.exe'
$psi = New-Object System.Diagnostics.ProcessStartInfo $tool, 'hold 150'
$psi.RedirectStandardOutput = $true
$psi.UseShellExecute = $false
if ($env:LOAD) {
    $lp = New-Object System.Diagnostics.ProcessStartInfo (Resolve-Path $env:LOAD).Path, '--test-threads 4'
    $lp.UseShellExecute = $false; $lp.RedirectStandardOutput = $true; $lp.RedirectStandardError = $true
    $load = [System.Diagnostics.Process]::Start($lp); $load.ProcessorAffinity = [IntPtr]15
    "load: pid $($load.Id)"
}
$broker = [System.Diagnostics.Process]::Start($psi)
if ($env:LOAD) { $broker.ProcessorAffinity = [IntPtr]15 }
$kids = @()
while ($null -ne ($line = $broker.StandardOutput.ReadLine())) {
    "out: $line"
    if ($line.StartsWith('hold: children ')) {
        $kids = $line.Substring(15).Split(' ') | ForEach-Object { [int]($_.Split('=')[1]) }
        break
    }
}
$t0 = Get-Date
foreach ($s in $At) {
    $wait = $s - ((Get-Date) - $t0).TotalSeconds
    if ($wait -gt 0) { Start-Sleep -Milliseconds ([int]($wait * 1000)) }
    "--- t=$s s"
    & (Join-Path $PSScriptRoot 'threads.ps1') @kids
}
$broker.Kill()
if ($load -and -not $load.HasExited) { $load.Kill() }
