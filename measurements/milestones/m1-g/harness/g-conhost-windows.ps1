# M1 phase G: starts hg-launch-tool hold 15 and lists the broker's children and each
# child's own children with their private and working-set sizes (a conhost.exe per child
# shows a console-subsystem child). Usage: powershell -File g-conhost-windows.ps1
$p = Start-Process -FilePath "$env:USERPROFILE\izcal-m1g-ml\release\hg-launch-tool.exe" -ArgumentList "hold","15" -PassThru -WindowStyle Hidden -RedirectStandardOutput "$env:USERPROFILE\m1g-hold.out"
Start-Sleep -Seconds 5
$all = Get-CimInstance Win32_Process
$kids = $all | Where-Object { $_.ParentProcessId -eq $p.Id }
foreach ($k in $kids) {
  "child {0} {1}" -f $k.ProcessId, $k.Name
  $all | Where-Object { $_.ParentProcessId -eq $k.ProcessId } | ForEach-Object {
    $pr = Get-Process -Id $_.ProcessId
    "   grandchild {0} {1} private={2} KiB ws={3} KiB" -f $_.ProcessId, $_.Name, [int]($pr.PrivateMemorySize64/1024), [int]($pr.WorkingSet64/1024)
  }
}
Wait-Process -Id $p.Id
