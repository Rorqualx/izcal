# M1 phase G: as g-fresh-phases-windows.ps1, but each fresh child copy is first run once
# directly (it exits at argument parsing), to test whether the cost is per file's first
# execution. Usage: as g-fresh-phases-windows.ps1.
param([string]$Build, [string]$Out, [int]$N = 6)
$st = Get-MpComputerStatus -ErrorAction SilentlyContinue
"defender: RealTimeProtectionEnabled=$($st.RealTimeProtectionEnabled) AMRunningMode=$($st.AMRunningMode) IsTamperProtected=$($st.IsTamperProtected)"
$bins = "hg-launch-tool.exe","hg-renderer-main.exe","hg-net-main.exe"
$mp = Join-Path $env:ProgramFiles "Windows Defender\MpCmdRun.exe"
$lines = @("run`tkind`tpid`tside`tphase`tms")
for ($i = 1; $i -le $N; $i++) {
  $d = Join-Path $env:TEMP ("g-av-" + [guid]::NewGuid())
  New-Item -ItemType Directory $d | Out-Null
  foreach ($b in $bins) { Copy-Item (Join-Path $Build $b) $d }
  $t0 = Get-Date
  foreach ($b in "hg-renderer-main.exe","hg-net-main.exe") { $p = Start-Process -FilePath (Join-Path $d $b) -PassThru -WindowStyle Hidden; $p.WaitForExit(); $code = $p.ExitCode }
  $scan = ((Get-Date) - $t0).TotalMilliseconds
  "run $i pre-exec took {0:N0} ms (exit $LASTEXITCODE)" -f $scan
  & (Join-Path $d "hg-launch-tool.exe") measure-phases 1 | Select-Object -Skip 1 | ForEach-Object { $f = $_ -split "`t", 2; "$i`t$($f[1])" } | ForEach-Object { $lines += $_ }
  Remove-Item -Recurse -Force $d
}
[IO.File]::WriteAllLines($Out, [string[]]$lines)
