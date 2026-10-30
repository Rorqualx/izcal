# M1 phase G: measure-phases 1 from a fresh copy of the binaries per run (E-R12's
# fresh-build-directory fallback), on a measure-launch build.
# Usage: powershell -File g-fresh-phases-windows.ps1 -Build DIR -Out TSV [-N 10]
param([string]$Build, [string]$Out, [int]$N = 10)
$bins = "hg-launch-tool.exe","hg-renderer-main.exe","hg-net-main.exe"
$lines = @("run`tkind`tpid`tside`tphase`tms")
for ($i = 1; $i -le $N; $i++) {
  $d = Join-Path $env:TEMP ("g-phase-" + [guid]::NewGuid())
  New-Item -ItemType Directory $d | Out-Null
  foreach ($b in $bins) { Copy-Item (Join-Path $Build $b) $d }
  & (Join-Path $d "hg-launch-tool.exe") measure-phases 1 | Select-Object -Skip 1 | ForEach-Object { $f = $_ -split "`t", 2; "$i`t$($f[1])" } | ForEach-Object { $lines += $_ }
  Remove-Item -Recurse -Force $d
}
[IO.File]::WriteAllLines($Out, [string[]]$lines)
