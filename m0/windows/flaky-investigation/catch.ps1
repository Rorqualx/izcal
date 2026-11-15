# Runs the hg-launch-tool test binary on 4 CPUs until one run is still
# going at 45 s (an_idle_child_runs_two_threads waiting), then dumps the
# threads of every hg-net-main / hg-renderer-main process at 45 s and 85 s.
# Usage: powershell -NoProfile -File catch.ps1 EXE MAXRUNS
param([string]$Exe, [int]$Max)
$exe = (Resolve-Path $Exe).Path
$probe = Join-Path $PSScriptRoot 'threads.ps1'
for ($i = 1; $i -le $Max; $i++) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo $exe, '--nocapture'
    $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    $p.ProcessorAffinity = [IntPtr]15
    $o = $p.StandardOutput.ReadToEndAsync(); $e = $p.StandardError.ReadToEndAsync()
    $caught = $false
    if (-not $p.WaitForExit(45000)) {
        $caught = $true
        "run $i still running at 45 s"
        $kids = Get-Process hg-net-main, hg-renderer-main -ErrorAction SilentlyContinue | ForEach-Object Id
        "--- 45 s"; & $probe @kids
        Start-Sleep -Seconds 40
        "--- 85 s"; & $probe @kids
    }
    $p.WaitForExit()
    $line = ($e.Result + $o.Result) -split "`n" | Where-Object { $_ -match 'threads after|test result' }
    "run $i exit=$($p.ExitCode) $($line -join ' | ')"
    if ($caught -and $p.ExitCode -ne 0) { break }
}
