# Lists a process's threads with their Win32 start address (module+offset),
# age, state and wait reason. Usage: powershell -NoProfile -File threads.ps1 PID [PID...]
param([Parameter(ValueFromRemainingArguments = $true)][int[]]$Pids)
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class Thr {
    [DllImport("kernel32.dll", SetLastError = true)]
    static extern IntPtr OpenThread(uint access, bool inherit, uint id);
    [DllImport("kernel32.dll")]
    static extern bool CloseHandle(IntPtr h);
    [DllImport("ntdll.dll")]
    static extern int NtQueryInformationThread(IntPtr h, int cls, out IntPtr info, int len, IntPtr ret);
    // THREAD_QUERY_INFORMATION; class 9 = ThreadQuerySetWin32StartAddress.
    public static long Start(uint tid) {
        IntPtr h = OpenThread(0x0040, false, tid);
        if (h == IntPtr.Zero) return -Marshal.GetLastWin32Error();
        IntPtr a;
        int st = NtQueryInformationThread(h, 9, out a, IntPtr.Size, IntPtr.Zero);
        CloseHandle(h);
        return st == 0 ? a.ToInt64() : -(long)(uint)st;
    }
}
'@
foreach ($procId in $Pids) {
    $p = Get-Process -Id $procId -ErrorAction SilentlyContinue
    if (-not $p) { "pid $procId gone"; continue }
    $mods = $p.Modules | ForEach-Object {
        [pscustomobject]@{ Name = $_.ModuleName; Base = [int64]$_.BaseAddress; End = [int64]$_.BaseAddress + $_.ModuleMemorySize }
    }
    $now = Get-Date
    $page = [int]($now - $p.StartTime).TotalSeconds
    foreach ($t in $p.Threads) {
        $a = [Thr]::Start([uint32]$t.Id)
        $m = $mods | Where-Object { $a -ge $_.Base -and $a -lt $_.End } | Select-Object -First 1
        $off = if ($a -lt 0) { "err$a" } elseif ($m) { '{0}+0x{1:x}' -f $m.Name, ($a - $m.Base) } else { '0x{0:x}' -f $a }
        $age = try { [int]($now - $t.StartTime).TotalSeconds } catch { -1 }
        $wr = if ($t.ThreadState -eq 'Wait') { $t.WaitReason } else { '' }
        '{0} {1} tid={2} start={3} born=+{4}s state={5} {6} cpu={7:n3}s' -f $p.ProcessName, $procId, $t.Id, $off, ($page - $age), $t.ThreadState, $wr, $t.TotalProcessorTime.TotalSeconds
    }
}
