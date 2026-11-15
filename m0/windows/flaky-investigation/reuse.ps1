# Black-box probe of handle-value reuse. For each trial: duplicate an event
# handle (value V), close V, optionally move the thread to another CPU,
# then duplicate up to 256 more times looking for V again.
# Mode same: thread pinned to CPU 2 throughout. Mode move: closed on CPU 2,
# looked for on CPU 5. Mode free: no pinning.
param([string]$Mode, [int]$Trials = 2000)
$src = $null
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class R {
  [DllImport("kernel32")] static extern IntPtr CreateEventW(IntPtr a, bool m, bool i, IntPtr n);
  [DllImport("kernel32")] static extern IntPtr GetCurrentProcess();
  [DllImport("kernel32")] static extern IntPtr GetCurrentThread();
  [DllImport("kernel32")] static extern bool DuplicateHandle(IntPtr sp, IntPtr s, IntPtr tp, out IntPtr t, uint acc, bool inh, uint opt);
  [DllImport("kernel32")] static extern bool CloseHandle(IntPtr h);
  [DllImport("kernel32")] static extern UIntPtr SetThreadAffinityMask(IntPtr t, UIntPtr m);
  [DllImport("kernel32")] static extern uint GetCurrentProcessorNumber();
  static IntPtr Dup(IntPtr e) { IntPtr t; DuplicateHandle(GetCurrentProcess(), e, GetCurrentProcess(), out t, 0, false, 2); return t; }
  public static string Run(string mode, int trials, int limit) {
    IntPtr e = CreateEventW(IntPtr.Zero, true, false, IntPtr.Zero);
    int miss = 0, cpuChanged = 0, missChanged = 0; long firstHitSum = 0; int maxIdx = 0;
    for (int k = 0; k < trials; k++) {
      if (mode != "free") SetThreadAffinityMask(GetCurrentThread(), (UIntPtr)(1u << 2));
      IntPtr v = Dup(e);
      uint c0 = GetCurrentProcessorNumber();
      CloseHandle(v);
      if (mode == "move") SetThreadAffinityMask(GetCurrentThread(), (UIntPtr)(1u << 5));
      uint c1 = GetCurrentProcessorNumber();
      bool changed = c0 != c1; if (changed) cpuChanged++;
      var extra = new System.Collections.Generic.List<IntPtr>();
      int idx = -1;
      for (int i = 0; i < limit; i++) { IntPtr h = Dup(e); extra.Add(h); if (h == v) { idx = i; break; } }
      foreach (var h in extra) CloseHandle(h);
      if (idx < 0) { miss++; if (changed) missChanged++; } else { firstHitSum += idx; if (idx > maxIdx) maxIdx = idx; }
    }
    return String.Format("limit="+limit+" mode={0} trials={1} miss={2} cpuChanged={3} missWithCpuChange={4} maxIdx={5}", mode, trials, miss, cpuChanged, missChanged, maxIdx);
  }
}
'@
[R]::Run($Mode, $Trials, [int]$env:LIMIT)
