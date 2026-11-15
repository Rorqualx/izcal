$t = Join-Path $env:USERPROFILE 'izcal-flaky\target\debug\hg-launch-tool.exe'
$f = Join-Path $env:TEMP 'aff.txt'
Start-Process -WindowStyle Hidden cmd.exe -ArgumentList '/c', "start `"`" /b /affinity F `"$t`" hold 8 > `"$f`""
Start-Sleep 3
Get-Process hg-net-main, hg-renderer-main, hg-launch-tool | ForEach-Object { '{0} {1} aff=0x{2:x}' -f $_.ProcessName, $_.Id, [int64]$_.ProcessorAffinity }
