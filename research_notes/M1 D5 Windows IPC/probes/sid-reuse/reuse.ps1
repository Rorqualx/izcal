# The SID-reuse probe: does anything of a container's previous holder
# reach the next process that runs under the same AppContainer SID?
# Runs as a standard user. izp.exe (izp.cs) is the child.
param([string]$Dir = "$env:USERPROFILE\reuse", [int]$Cycles = 200)
$src = @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Text;
public static class R {
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int CreateAppContainerProfile(string n, string d, string s, IntPtr c, uint k, out IntPtr sid);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int DeleteAppContainerProfile(string n);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int DeriveAppContainerSidFromAppContainerName(string n, out IntPtr sid);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int GetAppContainerFolderPath(string sid, out IntPtr path);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool ConvertSidToStringSidW(IntPtr sid, out IntPtr s);
  [DllImport("advapi32.dll")] static extern bool EqualSid(IntPtr a, IntPtr b);
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool OpenProcessToken(IntPtr p, uint a, out IntPtr t);
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool GetTokenInformation(IntPtr t, int c, IntPtr b, int n, out int r);
  [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr OpenProcess(uint a, bool i, uint pid);
  [DllImport("kernel32.dll")] static extern bool ProcessIdToSessionId(uint pid, out uint s);
  [DllImport("kernel32.dll")] static extern uint GetCurrentProcessId();
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr l, int n, int f, ref IntPtr size);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr l, uint f, IntPtr attr, IntPtr val, IntPtr size, IntPtr prev, IntPtr ret);
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool CreateProcessW(string app, StringBuilder cmd, IntPtr pa, IntPtr ta, bool inh, uint flags, IntPtr env, string dir, IntPtr si, out PI pi);
  [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h, uint ms);
  [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr h, out uint c);
  [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr h, uint c);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr h);
  [DllImport("ole32.dll")] static extern void CoTaskMemFree(IntPtr p);
  [DllImport("ntdll.dll")] static extern int NtOpenDirectoryObject(out IntPtr h, uint a, ref OA oa);
  [DllImport("ntdll.dll")] static extern int NtOpenEvent(out IntPtr h, uint a, ref OA oa);
  [StructLayout(LayoutKind.Sequential)] public struct PI { public IntPtr hp, ht; public uint pid, tid; }
  [StructLayout(LayoutKind.Sequential)] struct US { public ushort Len, Max; public IntPtr Buf; }
  [StructLayout(LayoutKind.Sequential)] struct OA { public int Len; public IntPtr Root, Name; public uint Attr; public IntPtr Sd, Qos; }

  public static string Str(IntPtr sid) { IntPtr s; ConvertSidToStringSidW(sid, out s); return Marshal.PtrToStringUni(s); }
  public static IntPtr Create(string name) { IntPtr sid; int hr = CreateAppContainerProfile(name, name, name, IntPtr.Zero, 0, out sid); if (hr != 0) throw new Exception("create " + name + " hr=0x" + hr.ToString("x")); return sid; }
  public static int Delete(string name) { return DeleteAppContainerProfile(name); }
  public static IntPtr Derive(string name) { IntPtr sid; DeriveAppContainerSidFromAppContainerName(name, out sid); return sid; }
  public static string Folder(IntPtr sid) { IntPtr p; int hr = GetAppContainerFolderPath(Str(sid), out p); if (hr != 0) return null; string s = Marshal.PtrToStringUni(p); CoTaskMemFree(p); return s; }

  public static PI Launch(string exe, string args, IntPtr sid) {
    int siSize = 112; IntPtr si = Marshal.AllocHGlobal(siSize); for (int i = 0; i < siSize; i++) Marshal.WriteByte(si, i, 0);
    Marshal.WriteInt32(si, 0, siSize); IntPtr size = IntPtr.Zero; InitializeProcThreadAttributeList(IntPtr.Zero, 1, 0, ref size);
    IntPtr l = Marshal.AllocHGlobal(size); InitializeProcThreadAttributeList(l, 1, 0, ref size);
    IntPtr caps = Marshal.AllocHGlobal(24); for (int i = 0; i < 24; i++) Marshal.WriteByte(caps, i, 0); Marshal.WriteIntPtr(caps, 0, sid);
    UpdateProcThreadAttribute(l, 0, (IntPtr)0x00020009, caps, (IntPtr)24, IntPtr.Zero, IntPtr.Zero);
    Marshal.WriteIntPtr(si, 104, l);
    PI pi; if (!CreateProcessW(exe, new StringBuilder("\"" + exe + "\" " + args), IntPtr.Zero, IntPtr.Zero, false, 0x08000000 | 0x00080000, IntPtr.Zero, null, si, out pi))
      throw new Exception("CreateProcess err=" + Marshal.GetLastWin32Error());
    CloseHandle(pi.ht); return pi;
  }
  public static long Wait(PI p, uint ms) { if (WaitForSingleObject(p.hp, ms) != 0) return -1; uint c; GetExitCodeProcess(p.hp, out c); return c; }
  public static void Kill(PI p) { TerminateProcess(p.hp, 1); WaitForSingleObject(p.hp, 5000); CloseHandle(p.hp); }
  public static void KillPid(uint pid) { IntPtr h = OpenProcess(0x0001 | 0x00100000, false, pid); if (h != IntPtr.Zero) { TerminateProcess(h, 1); WaitForSingleObject(h, 5000); CloseHandle(h); } }

  static int Nt(string path, bool dir, out IntPtr h) {
    var b = Marshal.StringToHGlobalUni(path); var us = new US { Len = (ushort)(path.Length * 2), Max = (ushort)(path.Length * 2), Buf = b };
    IntPtr pus = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(US))); Marshal.StructureToPtr(us, pus, false);
    var oa = new OA { Len = Marshal.SizeOf(typeof(OA)), Name = pus, Attr = 0x40 };
    return dir ? NtOpenDirectoryObject(out h, 0x1, ref oa) : NtOpenEvent(out h, 0x100000, ref oa);
  }
  public static string NamedDir(IntPtr sid) { uint s; ProcessIdToSessionId(GetCurrentProcessId(), out s); return @"\Sessions\" + s + @"\AppContainerNamedObjects\" + Str(sid); }
  // "present", "gone" or the status.
  public static string DirState(IntPtr sid) { IntPtr h; int st = Nt(NamedDir(sid), true, out h); if (st == 0) { CloseHandle(h); return "present"; } if (st == unchecked((int)0xC0000034)) return "gone"; return "status=0x" + st.ToString("x8"); }
  public static IntPtr HoldEvent(IntPtr sid) { IntPtr h; int st = Nt(NamedDir(sid) + @"\izp-ev", false, out h); if (st != 0) throw new Exception("NtOpenEvent 0x" + st.ToString("x8")); return h; }

  // Processes of this user whose token is in the container `sid`.
  public static List<uint> InContainer(IntPtr sid) {
    var o = new List<uint>(); IntPtr buf = Marshal.AllocHGlobal(256);
    foreach (var p in Process.GetProcesses()) {
      IntPtr h = OpenProcess(0x1000, false, (uint)p.Id); if (h == IntPtr.Zero) continue;
      IntPtr t; if (OpenProcessToken(h, 8, out t)) { int r; if (GetTokenInformation(t, 31, buf, 256, out r)) { IntPtr s = Marshal.ReadIntPtr(buf); if (s != IntPtr.Zero && EqualSid(s, sid)) o.Add((uint)p.Id); } CloseHandle(t); }
      CloseHandle(h);
    }
    return o;
  }
}
'@
Add-Type -TypeDefinition $src -Language CSharp
$exe = "$Dir\izp.exe"
$names = @{1='event';2='mutex';4='section';8='pipe';16='file';32='registry'}
function Bits($c) { if ($c -lt 0) { return "TIMEOUT" }; $f = @(); $o = @(); foreach ($k in 1,2,4,8,16,32) { if ($c -band $k) { $f += $names[$k] }; if ($c -band ($k -shl 8)) { $o += $names[$k] } }
  $s = "found=[" + ($f -join ',') + "]"; if ($o) { $s += " ERRORS=[" + ($o -join ',') + "]" }; "$s (code $c)" }
function NewName { "izcal.reuse." + [guid]::NewGuid().ToString('N').Substring(0, 10) }
function Run($sid, $folder, $a, $extra = "", $ms = 30000) { $p = [R]::Launch($exe, "$a `"$folder`" $extra", $sid); $c = [R]::Wait($p, $ms); [void][R]::CloseHandle($p.hp); $c }
function Check($sid, $folder) { Bits (Run $sid $folder 'check') }
function WaitClear($sid) { $t = [Diagnostics.Stopwatch]::StartNew(); while ($t.Elapsed.TotalSeconds -lt 30) { if (([R]::InContainer($sid)).Count -eq 0 -and [R]::DirState($sid) -eq 'gone') { return "clear after {0:N2} s" -f $t.Elapsed.TotalSeconds }; Start-Sleep -Milliseconds 100 }
  "NOT clear after 30 s: procs=" + (([R]::InContainer($sid)) -join ',') + " dir=" + [R]::DirState($sid) }
function Recycle($name) { $d = [R]::Delete($name); $sid = [R]::Create($name); "delete hr=0x{0:x}, recreated" -f $d; $script:sid = $sid; $script:folder = [R]::Folder($sid) }

"reuse probe $(Get-Date -Format s) pid=$PID user=$(whoami) build=$([Environment]::OSVersion.Version)"
$n = NewName; $a = [R]::Create($n); $b = [R]::Derive($n)
"SID is a function of the name: create=$([R]::Str($a)) derive=$([R]::Str($b)) equal=$([R]::Str($a) -eq [R]::Str($b))"
[void][R]::Delete($n); $c = [R]::Create($n); "after delete+create the same name: $([R]::Str($c)) equal=$([R]::Str($a) -eq [R]::Str($c))"; [void][R]::Delete($n)

"`n== C control: A planted and still running, B checks (the detector must find all six)"
$n = NewName; $sid = [R]::Create($n); $folder = [R]::Folder($sid); "folder $folder"
$p = [R]::Launch($exe, "plant `"$folder`" stay", $sid); Start-Sleep 3
"procs in container: " + ((([R]::InContainer($sid)) | ForEach-Object { "$_=" + (Get-Process -Id $_).ProcessName }) -join ',') + "; dir " + [R]::DirState($sid)
"B: " + (Check $sid $folder); [R]::Kill($p); [void][R]::Delete($n)

"`n== S1 recycle: A plants and exits; wait clear; delete+recreate profile; B checks (want none)"
$n = NewName; $sid = [R]::Create($n); $folder = [R]::Folder($sid)
"A plant: exit " + (Run $sid $folder 'plant'); WaitClear $sid
"folder before recycle: " + ((Get-ChildItem $folder -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object Name) -join ',')
Recycle $n; "folder after recycle: exists=$(Test-Path $folder) files=" + ((Get-ChildItem $folder -Recurse -File -ErrorAction SilentlyContinue | ForEach-Object Name) -join ',')
"B: " + (Check $sid $folder); [void][R]::Delete($n)

"`n== S2 same profile kept (no delete): A plants and exits; wait clear; B checks"
$n = NewName; $sid = [R]::Create($n); $folder = [R]::Folder($sid)
"A plant: exit " + (Run $sid $folder 'plant'); WaitClear $sid
"B: " + (Check $sid $folder); [void][R]::Delete($n)

"`n== S3 orphan: A plants, starts a grandchild in the container, exits"
$n = NewName; $sid = [R]::Create($n); $folder = [R]::Folder($sid)
"A plant: exit " + (Run $sid $folder 'plant' 'orphan'); Start-Sleep 1; $left = [R]::InContainer($sid)
"procs in container after A: " + (($left | ForEach-Object { "$_=" + (Get-Process -Id $_).ProcessName }) -join ',') + "; dir " + [R]::DirState($sid)
"B reusing now (rule broken): " + (Check $sid $folder)
foreach ($q in $left) { [R]::KillPid($q) }; WaitClear $sid; Recycle $n
"B after the grandchild ends, recycle: " + (Check $sid $folder); [void][R]::Delete($n)

"`n== S4 broker keeps a handle to A's event: A plants (stays), broker opens izp-ev, A killed"
$n = NewName; $sid = [R]::Create($n); $folder = [R]::Folder($sid)
$p = [R]::Launch($exe, "plant `"$folder`" stay", $sid); Start-Sleep 3; $h = [IntPtr]::Zero; try { $h = [R]::HoldEvent($sid); "broker holds izp-ev" } catch { "HOLD FAILED: $_" }; [R]::Kill($p); Start-Sleep 1
"procs in container: " + (([R]::InContainer($sid)) -join ',') + "; dir " + [R]::DirState($sid)
Recycle $n; "B (event still held by the broker): " + (Check $sid $folder)
if ($h -ne [IntPtr]::Zero) { [void][R]::CloseHandle($h) }; WaitClear $sid; "B after the broker closes it: " + (Check $sid $folder); [void][R]::Delete($n)

"`n== M SeLs cost of reuse ($Cycles cycles each)"
$tags = 'SeLs'
function SeLs { Start-Sleep 10; $f = "$Dir\pool.tsv"; & "$env:USERPROFILE\trace\pool.ps1" -Top 1 -Dump $f | Out-Null
  foreach ($l in Get-Content $f) { $x = $l -split "`t"; if ($x[0] -ceq 'SeLs') { return [int64]$x[1] + [int64]$x[3] } } }
$s0 = SeLs; $n = NewName; $ok = 0
for ($i = 0; $i -lt $Cycles; $i++) { $sid = [R]::Create($n); $folder = [R]::Folder($sid); if ((Run $sid $folder 'check') -eq 0) { $ok++ }; [void][R]::Delete($n) }
$s1 = SeLs; "one name, delete+recreate+launch x$Cycles (clean checks $ok): SeLs $s0 -> $s1 (+$($s1 - $s0))"
$ok = 0
for ($i = 0; $i -lt $Cycles; $i++) { $m = NewName; $sid = [R]::Create($m); $folder = [R]::Folder($sid); if ((Run $sid $folder 'check') -eq 0) { $ok++ }; [void][R]::Delete($m) }
$s2 = SeLs; "control, fresh name each x$Cycles (clean checks $ok): SeLs $s1 -> $s2 (+$($s2 - $s1))"
$s3 = SeLs; "idle 10 s: SeLs $s2 -> $s3 (+$($s3 - $s2))"
