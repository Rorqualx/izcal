# Runs a test binary N times on 4 CPUs and prints each run's result line.
# Usage: powershell -NoProfile -File loop.ps1 EXE N [test-args...]
param([string]$Exe, [int]$N, [Parameter(ValueFromRemainingArguments = $true)][string[]]$Rest)
$exe = (Resolve-Path $Exe).Path
for ($i = 1; $i -le $N; $i++) {
    $psi = New-Object System.Diagnostics.ProcessStartInfo $exe, ($Rest -join ' ')
    $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true; $psi.RedirectStandardError = $true
    $p = [System.Diagnostics.Process]::Start($psi)
    $p.ProcessorAffinity = [IntPtr]$(if ($env:AFF) { [int]$env:AFF } else { 15 })
    $err = $p.StandardError.ReadToEndAsync()
    $out = $p.StandardOutput.ReadToEnd()
    $p.WaitForExit()
    $res = ($out -split "`n" | Where-Object { $_ -match 'test result|FAILED|threads after|D6-STALL|panicked|left:|no regions' }) -join ' | '
    $e = ($err.Result -split "`n" | Where-Object { $_ -match 'threads after' }) -join ' '
    "run $i exit=$($p.ExitCode) $res $e"
}
