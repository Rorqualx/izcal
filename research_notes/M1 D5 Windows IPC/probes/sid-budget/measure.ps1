$t = 'C:\Users\izcal-run\trace'
. "$t\acloop.ps1"
$tags = 'SeLs','Toke','SeTd','ObDi','Proc','SeAt'
function Snap($label) { Start-Sleep 10; & "$t\pool.ps1" -Top 1 -Dump "$t\m-$label.tsv" | Out-Null
  $h = @{}; foreach ($l in Get-Content "$t\m-$label.tsv") { $f = $l -split "`t"; if ($tags -ccontains $f[0]) { $h[$f[0]] = [int64]$f[1] + [int64]$f[3] } }; $h }
function Show($label, $a, $b) { $s = "$label :"; foreach ($k in $tags) { $s += " {0}={1:+#;-#;0}" -f $k, ($b[$k] - $a[$k]) }; $s }
$s0 = Snap 0; "start: " + (($tags | ForEach-Object { "$_=$($s0[$_])" }) -join ' ')
"A fresh profiles: " + [AC]::Loop('C:\Windows\System32\cmd.exe', 300, $true); $s1 = Snap 1; Show 'A delta (300 fresh)' $s0 $s1
"B one profile: " + [AC]::Loop('C:\Windows\System32\cmd.exe', 300, $false); $s2 = Snap 2; Show 'B delta (300 same)' $s1 $s2
$ok = 0; for ($i = 0; $i -lt 20; $i++) { $o = & "$env:USERPROFILE\izcal\target\debug\hg-launch-tool.exe" round-trip 2>&1 | Out-String; if ($o -match 'round-trip: ok') { $ok++ } }
"C round-trips ok=$ok/20"; $s3 = Snap 3; Show 'C delta (20 round-trips)' $s2 $s3
