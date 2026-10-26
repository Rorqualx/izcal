# M1 phase G, Windows: the renderer-skeleton row (and the network
# partition service beside it), as g-run-mac.sh does on macOS.
# - Charged memory: commit charge, Process.PrivateMemorySize64 (FP 6.2's
#   Windows metric), sampled at 1 Hz over a 60 s quiet window; settled is
#   the median of the last 30 s; 5 runs.
# - Idle wakeups: the raw cumulative "Context Switches" of every thread
#   (Win32_PerfRawData_PerfProc_Thread), FP-M0's wakeups-windows.ps1
#   method, over the same window. The window starts 40 s after `ready`,
#   as FP-M0's Windows runs did, so the thread set is stable.
# - Mappings: VirtualQueryEx over the user address space; allocations
#   (distinct AllocationBase of committed or reserved regions, one VAD
#   each) and regions, FP 6.20.6.
# Usage: powershell -File g-run-windows.ps1 -Build DIR -Out TSV [-Runs 5]
param([string]$Build, [string]$Out, [int]$Runs = 5)
Add-Type -TypeDefinition @"
using System;
using System.Runtime.InteropServices;
public static class Vq {
  [StructLayout(LayoutKind.Sequential)]
  public struct MBI { public IntPtr BaseAddress; public IntPtr AllocationBase; public uint AllocationProtect; public ushort PartitionId; public IntPtr RegionSize; public uint State; public uint Protect; public uint Type; }
  [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr OpenProcess(uint a, bool i, int pid);
  [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr VirtualQueryEx(IntPtr h, IntPtr addr, out MBI m, IntPtr len);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  // Returns { allocations, regions } of committed or reserved memory.
  public static long[] Count(int pid) {
    IntPtr h = OpenProcess(0x0400 | 0x0010, false, pid); // QUERY_INFORMATION | VM_READ
    if (h == IntPtr.Zero) return new long[] { -1, -1 };
    var bases = new System.Collections.Generic.HashSet<long>();
    long regions = 0; long addr = 0; MBI m;
    while (VirtualQueryEx(h, new IntPtr(addr), out m, new IntPtr(Marshal.SizeOf(typeof(MBI)))) != IntPtr.Zero) {
      if (m.State != 0x10000) { regions++; bases.Add(m.AllocationBase.ToInt64()); } // not MEM_FREE
      long next = m.BaseAddress.ToInt64() + m.RegionSize.ToInt64();
      if (next <= addr) break; addr = next;
    }
    CloseHandle(h);
    return new long[] { bases.Count, regions };
  }
}
"@
function Switches($procName) {
  $t = @{}
  Get-CimInstance Win32_PerfRawData_PerfProc_Thread -Filter "Name LIKE '$procName/%'" |
    ForEach-Object { $t["$($_.IDThread)"] = [int64]$_.ContextSwitchesPersec }
  $t
}
function Delta($a, $b) { $s = 0; foreach ($k in $b.Keys) { if ($a.ContainsKey($k)) { $s += $b[$k] - $a[$k] } }; $s }
function Median($xs) { $s = @($xs | Sort-Object); $n = $s.Count; if ($n % 2) { $s[($n-1)/2] } else { ($s[$n/2-1] + $s[$n/2]) / 2 } }
$rows = @("run`tkind`tpid`tsettled_kb`tpeak_kb`tswitches_per_s`tthreads`tallocations`tregions")
for ($run = 1; $run -le $Runs; $run++) {
  $log = [IO.Path]::GetTempFileName()
  $tool = Start-Process -FilePath (Join-Path $Build "hg-launch-tool.exe") -ArgumentList "hold","140" -RedirectStandardOutput $log -NoNewWindow -PassThru
  for ($i = 0; $i -lt 200 -and -not (Select-String -Path $log -Pattern "^hold: ready" -Quiet); $i++) { Start-Sleep -Milliseconds 100 }
  if (-not (Select-String -Path $log -Pattern "^hold: ready" -Quiet)) { Get-Content $log; throw "run ${run}: no ready" }
  $kids = Get-CimInstance Win32_Process -Filter "ParentProcessId = $($tool.Id)"
  $procs = @{ renderer = ($kids | Where-Object Name -eq "hg-renderer-main.exe").ProcessId; net = ($kids | Where-Object Name -eq "hg-net-main.exe").ProcessId }
  Start-Sleep -Seconds 40
  $sw0 = @{ renderer = Switches "hg-renderer-main"; net = Switches "hg-net-main" }
  $mem = @{ renderer = @(); net = @() }
  for ($t = 0; $t -lt 60; $t++) {
    foreach ($k in @("renderer","net")) { $mem[$k] += [math]::Round((Get-Process -Id $procs[$k]).PrivateMemorySize64 / 1KB) }
    Start-Sleep -Seconds 1
  }
  $sw1 = @{ renderer = Switches "hg-renderer-main"; net = Switches "hg-net-main" }
  foreach ($k in @("renderer","net")) {
    $settled = Median ($mem[$k] | Select-Object -Last 30)
    $peak = ($mem[$k] | Measure-Object -Maximum).Maximum
    $rate = "{0:N3}" -f ((Delta $sw0[$k] $sw1[$k]) / 60)
    $c = [Vq]::Count([int]$procs[$k])
    $rows += "$run`t$k`t$($procs[$k])`t$settled`t$peak`t$rate`t$($sw1[$k].Count)`t$($c[0])`t$($c[1])"
  }
  $tool.WaitForExit()
  Remove-Item $log
}
[IO.File]::WriteAllLines($Out, $rows)
$rows
