param([string]$Exe = 'C:\Users\izcal-run\izcal\target\debug\hg-renderer-main.exe')
$src = @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class AC {
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int CreateAppContainerProfile(string name, string disp, string desc, IntPtr caps, uint n, out IntPtr sid);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int DeleteAppContainerProfile(string name);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr l, int n, int f, ref IntPtr size);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr l, uint f, IntPtr attr, IntPtr val, IntPtr size, IntPtr prev, IntPtr ret);
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool CreateProcessW(string app, StringBuilder cmd, IntPtr pa, IntPtr ta, bool inh, uint flags, IntPtr env, string dir, IntPtr si, out PI pi);
  [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr h, uint c);
  [DllImport("kernel32.dll")] static extern bool CloseHandle(IntPtr h);
  [StructLayout(LayoutKind.Sequential)] public struct PI { public IntPtr hp, ht; public uint pid, tid; }
  static string Try(string exe, IntPtr sid, bool useAttr) {
    int siSize = 112; IntPtr si = Marshal.AllocHGlobal(siSize); for (int i = 0; i < siSize; i++) Marshal.WriteByte(si, i, 0);
    uint flags = 0x08000000 | 0x4; // CREATE_NO_WINDOW | CREATE_SUSPENDED
    if (useAttr) {
      Marshal.WriteInt32(si, 0, siSize); IntPtr size = IntPtr.Zero; InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
      IntPtr l = Marshal.AllocHGlobal(size); if (!InitializeProcThreadAttributeList(l, 1, 0, ref size)) return "init err " + Marshal.GetLastWin32Error();
      IntPtr caps = Marshal.AllocHGlobal(24); for (int i = 0; i < 24; i++) Marshal.WriteByte(caps, i, 0); Marshal.WriteIntPtr(caps, 0, sid);
      if (!UpdateProcThreadAttribute(l, 0, (IntPtr)0x00020009, caps, (IntPtr)24, IntPtr.Zero, IntPtr.Zero)) return "update err " + Marshal.GetLastWin32Error();
      Marshal.WriteIntPtr(si, 104, l); flags |= 0x00080000; // EXTENDED_STARTUPINFO_PRESENT
    } else Marshal.WriteInt32(si, 0, 104);
    PI pi; bool ok = CreateProcessW(exe, new StringBuilder("\"" + exe + "\""), IntPtr.Zero, IntPtr.Zero, false, flags, IntPtr.Zero, null, si, out pi);
    if (!ok) return "FAIL err=" + Marshal.GetLastWin32Error();
    TerminateProcess(pi.hp, 0); CloseHandle(pi.hp); CloseHandle(pi.ht); return "ok pid=" + pi.pid;
  }
  public static string Run(string exe) {
    var o = new StringBuilder();
    o.AppendLine("plain: " + Try(exe, IntPtr.Zero, false));
    string name = "izcal.acprobe." + Guid.NewGuid().ToString("N").Substring(0, 8); IntPtr sid;
    int hr = CreateAppContainerProfile(name, name, name, IntPtr.Zero, 0, out sid); o.AppendLine("profile " + name + " hr=0x" + hr.ToString("x"));
    if (hr == 0) { o.AppendLine("appcontainer: " + Try(exe, sid, true)); o.AppendLine("appcontainer again: " + Try(exe, sid, true)); o.AppendLine("appcontainer cmd.exe: " + Try(@"C:\Windows\System32\cmd.exe", sid, true)); o.AppendLine("delete hr=0x" + DeleteAppContainerProfile(name).ToString("x")); }
    return o.ToString();
  }
}
'@
Add-Type -TypeDefinition $src -Language CSharp
"probe pid=$PID"; [AC]::Run($Exe)
