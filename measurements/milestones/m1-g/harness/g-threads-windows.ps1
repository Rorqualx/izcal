# M1 phase G: lists each launched child's threads by name. Usage: powershell -File g-threads-windows.ps1 -Build DIR
# M1 phase G: starts hg-launch-tool hold 20 from -Build and lists each child's threads
# with their descriptions (names), via OpenThread + GetThreadDescription.
param([string]$Build)
Add-Type -Namespace M1G -Name T -MemberDefinition @'
[DllImport("kernel32.dll")] public static extern IntPtr OpenThread(uint access, bool inherit, uint id);
[DllImport("kernel32.dll")] public static extern int GetThreadDescription(IntPtr h, out IntPtr desc);
[DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr h);
[DllImport("kernel32.dll")] public static extern IntPtr LocalFree(IntPtr p);
'@
$p = Start-Process -FilePath (Join-Path $Build "hg-launch-tool.exe") -ArgumentList "hold","20" -PassThru -WindowStyle Hidden
Start-Sleep -Seconds 6
$kids = Get-CimInstance Win32_Process | Where-Object { $_.ParentProcessId -eq $p.Id -and $_.Name -like 'hg-*' }
foreach ($k in $kids) {
  $proc = Get-Process -Id $k.ProcessId
  "{0} pid={1} threads={2} private={3} KiB" -f $k.Name, $k.ProcessId, $proc.Threads.Count, [int]($proc.PrivateMemorySize64/1024)
  foreach ($t in $proc.Threads) {
    $h = [M1G.T]::OpenThread(0x0800, $false, [uint32]$t.Id)
    $name = ''
    if ($h -ne [IntPtr]::Zero) { $d = [IntPtr]::Zero; if ([M1G.T]::GetThreadDescription($h, [ref]$d) -ge 0 -and $d -ne [IntPtr]::Zero) { $name = [Runtime.InteropServices.Marshal]::PtrToStringUni($d); [void][M1G.T]::LocalFree($d) }; [void][M1G.T]::CloseHandle($h) }
    "   tid={0} state={1} wait={2} name='{3}'" -f $t.Id, $t.ThreadState, $t.WaitReason, $name
  }
}
Wait-Process -Id $p.Id
