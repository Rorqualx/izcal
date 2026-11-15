$a = [datetime]'2026-11-07 00:13:25'; $b = [datetime]'2026-11-07 00:13:46'
Get-WinEvent -ListLog * -ErrorAction SilentlyContinue | Where-Object { $_.RecordCount -gt 0 } | ForEach-Object {
  $n = $_.LogName
  try { Get-WinEvent -FilterHashtable @{ LogName = $n; StartTime = $a; EndTime = $b } -ErrorAction Stop | ForEach-Object { '{0:HH:mm:ss.fff} [{1}] {2} id={3} {4}' -f $_.TimeCreated, $n, $_.ProviderName, $_.Id, (($_.Message -split "`n")[0]) } } catch {}
} | Sort-Object
