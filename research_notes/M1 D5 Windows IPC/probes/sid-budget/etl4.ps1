# Native ETL reader (OpenTrace/ProcessTrace) for the kernel trace around one moment.
# Prints: file span, thread-creation bursts, the thread starts near $T with their
# creation stacks, who readied each creator, the top readiers, and other providers.
param([string]$Etl = 'C:\Users\izcal-run\trace\sc.etl', [int]$Pid0 = 0,
      [string]$T = '2026-11-07 20:35:33.180',
      [string]$Out = 'C:\Users\izcal-run\trace\etl4.txt')
$src = @'
using System; using System.Collections.Generic; using System.Runtime.InteropServices; using System.Text;
public static class Etl4 {
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern ulong OpenTraceW(IntPtr lf);
  [DllImport("advapi32.dll")] static extern int ProcessTrace(ulong[] h, uint n, IntPtr s, IntPtr e);
  [DllImport("advapi32.dll")] static extern int CloseTrace(ulong h);
  [UnmanagedFunctionPointer(CallingConvention.StdCall)] delegate void Cb(IntPtr rec);
  static Cb keep;
  class Mod { public ulong b, s; public string n; }
  class Ev { public long ts; public int kind; public uint pid, tid, hpid, htid, target; public int cpu; public ulong addr; public ulong[] stack; public string extra; }
  static Dictionary<uint, List<Mod>> mods = new Dictionary<uint, List<Mod>>();
  static Dictionary<uint, string> pname = new Dictionary<uint, string>();
  static Dictionary<uint, uint> tpid = new Dictionary<uint, uint>();
  static List<Ev> starts = new List<Ev>(), readies = new List<Ev>(), others = new List<Ev>();
  static Ev[] lastOnCpu = new Ev[256]; static uint[] cur = new uint[256];
  static Dictionary<long, int> bins = new Dictionary<long, int>();
  static long first = 0, last = 0, total = 0, w1a, w1b, w2a, w2b;
  static long gF0, gF1; public static uint target; static Dictionary<uint, ulong> pend = new Dictionary<uint, ulong>(); static Dictionary<string, int> fails = new Dictionary<string, int>(); static Dictionary<string, int> calls = new Dictionary<string, int>(); static List<string> seq = new List<string>();
  static long gT0, gT1, gP0, gP1, gI0, gI1, gS0, gS1;
  static void G(string s, out long a, out long b) { var x = new Guid(s).ToByteArray(); a = BitConverter.ToInt64(x, 0); b = BitConverter.ToInt64(x, 8); }
  static string Str(IntPtr p, int max, bool wide) { var sb = new StringBuilder(); for (int i = 0; i < max; ) { int c = wide ? Marshal.ReadInt16(p, i) : Marshal.ReadByte(p, i); if (c == 0) break; sb.Append((char)(c & 0xffff)); i += wide ? 2 : 1; } return sb.ToString(); }
  static void OnEvent(IntPtr r) {
    try {
      long ts = Marshal.ReadInt64(r, 16); total++;
      if (first == 0) first = ts; last = ts;
      long g0 = Marshal.ReadInt64(r, 24), g1 = Marshal.ReadInt64(r, 32);
      int op = Marshal.ReadByte(r, 45), ver = Marshal.ReadByte(r, 42), cpu = Marshal.ReadByte(r, 80);
      uint htid = (uint)Marshal.ReadInt32(r, 8), hpid = (uint)Marshal.ReadInt32(r, 12);
      int len = (ushort)Marshal.ReadInt16(r, 86); IntPtr ud = Marshal.ReadIntPtr(r, 96);
      bool in1 = ts >= w1a && ts <= w1b, in2 = ts >= w2a && ts <= w2b;
      if (g0 == gT0 && g1 == gT1) {
        if (op == 36) { cur[cpu] = (uint)Marshal.ReadInt32(ud, 0); return; }
        if (op == 50) {
          if (!in2) { lastOnCpu[cpu] = null; return; }
          var e = new Ev { ts = ts, kind = 50, target = (uint)Marshal.ReadInt32(ud, 0), htid = htid, hpid = hpid, cpu = cpu, tid = cur[cpu], extra = "adj=" + (sbyte)Marshal.ReadByte(ud, 4) + " inc=" + (sbyte)Marshal.ReadByte(ud, 5) };
          readies.Add(e); lastOnCpu[cpu] = e; return;
        }
        if (op >= 1 && op <= 4 && len >= 56) {
          uint pid = (uint)Marshal.ReadInt32(ud, 0), tid = (uint)Marshal.ReadInt32(ud, 4); tpid[tid] = pid;
          if (op == 1) {
            long bin = ts / 1000000; int c; bins.TryGetValue(bin, out c); bins[bin] = c + 1;
            if (in1) { var e = new Ev { ts = ts, kind = 1, pid = pid, tid = tid, htid = htid, hpid = hpid, cpu = cpu, addr = (ulong)Marshal.ReadInt64(ud, 48) }; starts.Add(e); lastOnCpu[cpu] = e; }
            else lastOnCpu[cpu] = null;
          }
          if (op == 2 && in1) starts.Add(new Ev { ts = ts, kind = 2, pid = pid, tid = tid, htid = htid, hpid = hpid, cpu = cpu });
          return;
        }
        return;
      }
      if (g0 == gF0 && g1 == gF1 && (op == 51 || op == 52)) {
        uint t = cur[cpu]; uint p; if (!tpid.TryGetValue(t, out p) || p != target) return;
        if (op == 51) { pend[t] = (ulong)Marshal.ReadInt64(ud, 0); return; }
        ulong a; if (!pend.TryGetValue(t, out a)) return; pend.Remove(t);
        uint stc = (uint)Marshal.ReadInt32(ud, 0); string k = "0x" + a.ToString("x") + " st=0x" + stc.ToString("x8");
        int c; calls.TryGetValue("0x" + a.ToString("x"), out c); calls["0x" + a.ToString("x")] = c + 1;
        if (stc >= 0xC0000000) { fails.TryGetValue(k, out c); fails[k] = c + 1; if (seq.Count < 300) seq.Add(T(ts) + " tid=" + t + " " + k); }
        return;
      }
      if (g0 == gS0 && g1 == gS1) {
        if (op != 32 || len < 24) return;
        var e = lastOnCpu[cpu]; if (e == null || e.stack != null) return;
        if (ts - e.ts > 20000) return;
        int n = (len - 16) / 8; var st = new ulong[n]; for (int i = 0; i < n; i++) st[i] = (ulong)Marshal.ReadInt64(ud, 16 + 8 * i);
        e.stack = st; e.extra = (e.extra ?? "") + " stk=" + (uint)Marshal.ReadInt32(ud, 8) + "/" + (uint)Marshal.ReadInt32(ud, 12); return;
      }
      if (g0 == gI0 && g1 == gI1) {
        if ((op == 10 || op == 3 || op == 4) && len > 58) {
          uint pid = (uint)Marshal.ReadInt32(ud, 16); string fn = Str(ud + 56, len - 56, true); int k = fn.LastIndexOf('\\'); if (k >= 0) fn = fn.Substring(k + 1);
          List<Mod> l; if (!mods.TryGetValue(pid, out l)) { l = new List<Mod>(); mods[pid] = l; }
          l.Add(new Mod { b = (ulong)Marshal.ReadInt64(ud, 0), s = (ulong)Marshal.ReadInt64(ud, 8), n = fn });
        }
        return;
      }
      if (g0 == gP0 && g1 == gP1) {
        if ((op == 1 || op == 3 || op == 4) && len > 40) {
          uint pid = (uint)Marshal.ReadInt32(ud, 8); int off = ver >= 4 ? 36 : 32;
          try { long u = Marshal.ReadInt64(ud, off); off += 16; int cnt = Marshal.ReadByte(ud, off + 1); off += 8 + 4 * cnt; pname[pid] = Str(ud + off, Math.Max(0, len - off), false); } catch { }
        }
        return;
      }
      if (in1 && others.Count < 400) others.Add(new Ev { ts = ts, kind = Marshal.ReadInt16(r, 40), hpid = hpid, htid = htid, cpu = cpu, extra = new Guid(BitConverter.GetBytes(g0).Concat8(BitConverter.GetBytes(g1))).ToString() + " op=" + op + " task=" + (ushort)Marshal.ReadInt16(r, 46) + " len=" + len });
    } catch { }
  }
  static byte[] Concat8(this byte[] a, byte[] b) { var x = new byte[16]; Array.Copy(a, x, 8); Array.Copy(b, 0, x, 8, 8); return x; }
  static string P(uint pid) { string n; return pid + "(" + (pname.TryGetValue(pid, out n) ? n : "?") + ")"; }
  static string Sym(uint pid, ulong a) {
    List<Mod> l; uint p = a >= 0xFFFF800000000000UL ? 0 : pid;
    if (mods.TryGetValue(p, out l)) foreach (var m in l) if (a >= m.b && a < m.b + m.s) return m.n + "+0x" + (a - m.b).ToString("x");
    if (p == 0) foreach (var kv in mods) foreach (var m in kv.Value) if (a >= m.b && a < m.b + m.s) return m.n + "+0x" + (a - m.b).ToString("x");
    return "0x" + a.ToString("x");
  }
  static string T(long ts) { return DateTime.FromFileTime(ts).ToString("HH:mm:ss.ffffff"); }
  static void Stack(StringBuilder o, Ev e, uint pid, int max) {
    if (e.stack == null) { o.AppendLine("      (no stack)"); return; }
    for (int i = 0; i < e.stack.Length && i < max; i++) o.AppendLine("      " + Sym(pid, e.stack[i]));
  }
  public static string Run(string path, long t) {
    G("3d6fa8d1-fe05-11d0-9dda-00c04fd7ba7c", out gT0, out gT1); G("ce1dbfb4-137e-4da6-87b0-3f59aa102cbc", out gF0, out gF1); G("3d6fa8d0-fe05-11d0-9dda-00c04fd7ba7c", out gP0, out gP1);
    G("2cb15d1d-5fc1-11d2-abe1-00a0c911f518", out gI0, out gI1); G("def2fe46-7bd6-4b80-bd94-f57fe20d0ce3", out gS0, out gS1);
    w1a = t - 15000000; w1b = t + 15000000; w2a = t - 1500000; w2b = t + 1000000;
    IntPtr lf = Marshal.AllocHGlobal(0x1C0); for (int i = 0; i < 0x1C0; i++) Marshal.WriteByte(lf, i, 0);
    IntPtr name = Marshal.StringToHGlobalUni(path);
    Marshal.WriteIntPtr(lf, 0, name); Marshal.WriteInt32(lf, 28, 0x10000000);
    keep = OnEvent; Marshal.WriteIntPtr(lf, 424, Marshal.GetFunctionPointerForDelegate(keep));
    ulong h = OpenTraceW(lf); if (h == ulong.MaxValue) return "OpenTrace failed " + Marshal.GetLastWin32Error();
    int rc = ProcessTrace(new ulong[] { h }, 1, IntPtr.Zero, IntPtr.Zero); CloseTrace(h);
    var o = new StringBuilder();
    o.AppendLine("rc=" + rc + " events=" + total + " first=" + T(first) + " last=" + T(last) + " lost=" + Marshal.ReadInt32(lf, 416) + " procs named=" + pname.Count + " modpids=" + mods.Count);
    List<Mod> km; ulong kb = 0; if (mods.TryGetValue(0, out km)) foreach (var m in km) if (m.n.ToLower() == "ntoskrnl.exe") kb = m.b;
    o.AppendLine("target=" + P(target) + " ntoskrnl base=0x" + kb.ToString("x") + " distinct syscalls=" + calls.Count);
    o.AppendLine("== failing syscalls (addr status count)"); foreach (var kv in fails) o.AppendLine("  " + kv.Key + " n=" + kv.Value + " sym=" + Sym(0, ulong.Parse(kv.Key.Substring(2, kv.Key.IndexOf(' ') - 2), System.Globalization.NumberStyles.HexNumber)));
    o.AppendLine("== sequence"); foreach (var x in seq) o.AppendLine(x);
    return o.ToString();
  }
}
'@
Add-Type -TypeDefinition $src -Language CSharp
$t = ([DateTime]::Parse($T)).ToFileTime()
[Etl4]::target = $Pid0; $r = [Etl4]::Run($Etl, 0)
Set-Content $Out $r
"written $Out ($((Get-Item $Out).Length) bytes)"
