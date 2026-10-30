# Lists (and with -Delete unregisters) AppContainer profiles left by the M1 launcher's
# first naming scheme, izcal.<token>.<broker pid>.<n> (token izk1..izk9), when the owning
# broker is provably gone: no process holds that pid now, or the one holding it started
# after the profile's folder was created. Every other name is only listed.
param([switch]$Delete)
Add-Type -Namespace M1G -Name U2 -MemberDefinition '[DllImport("userenv.dll", CharSet=CharSet.Unicode)] public static extern int DeleteAppContainerProfile(string name);'
$dir = Join-Path $env:LOCALAPPDATA "Packages"
$rows = foreach ($f in Get-ChildItem $dir -Directory | Where-Object { $_.Name -like 'izcal*' }) {
  if ($f.Name -match '^izcal\.izk\d\.(\d+)\.\d+$') {
    $ownerPid = [int]$Matches[1]
    $p = Get-Process -Id $ownerPid -ErrorAction SilentlyContinue
    $gone = (-not $p) -or ($p.StartTime -gt $f.CreationTime)
    $why = if (-not $p) { "no process has pid $ownerPid" } elseif ($gone) { "pid $ownerPid now $($p.ProcessName), started after" } else { "pid $ownerPid alive since before: KEEP" }
    [pscustomobject]@{ Name=$f.Name; Scheme='m1-launcher-v0'; Delete=$gone; Why=$why }
  } else {
    [pscustomobject]@{ Name=$f.Name; Scheme='other'; Delete=$false; Why='not this scheme: KEEP' }
  }
}
$rows | Sort-Object Scheme,Name | ForEach-Object { "{0}`t{1}`t{2}`t{3}" -f $_.Scheme, $_.Delete, $_.Name, $_.Why }
if ($Delete) {
  $ok = 0; $bad = 0
  foreach ($r in $rows | Where-Object { $_.Delete }) { if ([M1G.U2]::DeleteAppContainerProfile($r.Name) -eq 0) { $ok++ } else { $bad++ } }
  "deleted=$ok failed=$bad"
}
