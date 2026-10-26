# M1 phase G, Windows: FP-27 spawn-to-ready on the full sandbox path, as
# g-spawn.sh: warm (measure-spawn N) or a fresh copy of the binaries per
# run in a new directory (E-R12's fallback for cold).
# Usage: powershell -File g-spawn-windows.ps1 -Build DIR -Mode warm|fresh -Out TSV [-N 20]
param([string]$Build, [string]$Mode, [string]$Out, [int]$N = 20)
$bins = "hg-launch-tool.exe","hg-renderer-main.exe","hg-net-main.exe"
if ($Mode -eq "warm") {
  $lines = & (Join-Path $Build "hg-launch-tool.exe") measure-spawn $N
} else {
  $lines = @("run`tkind`tspawn_to_ready_ms")
  for ($i = 1; $i -le $N; $i++) {
    $d = Join-Path $env:TEMP ("g-spawn-" + [guid]::NewGuid())
    New-Item -ItemType Directory $d | Out-Null
    foreach ($b in $bins) { Copy-Item (Join-Path $Build $b) $d }
    & (Join-Path $d "hg-launch-tool.exe") measure-spawn 1 | Select-Object -Skip 1 | ForEach-Object { $f = $_ -split "`t"; "$i`t$($f[1])`t$($f[2])" } | ForEach-Object { $lines += $_ }
    Remove-Item -Recurse -Force $d
  }
}
[IO.File]::WriteAllLines($Out, [string[]]$lines)
$lines | Select-Object -Skip 1 | ForEach-Object { $f = $_ -split "`t"; [pscustomobject]@{k=$f[1]; v=[double]$f[2]} } | Group-Object k | ForEach-Object {
  $s = @($_.Group.v | Sort-Object); $n = $s.Count
  $m = if ($n % 2) { $s[($n-1)/2] } else { ($s[$n/2-1] + $s[$n/2]) / 2 }
  "{0} n={1} median={2:N3} min={3:N3} max={4:N3}" -f $_.Name, $n, $m, $s[0], $s[$n-1]
}
