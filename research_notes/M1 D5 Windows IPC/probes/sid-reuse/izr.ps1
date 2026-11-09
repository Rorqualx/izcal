# izr.ps1: the interactive-session probe for the two izq channels that
# cannot run over ssh, where user32.dll fails its DllMain inside an
# AppContainer (0xC0000142). Start it INSIDE an interactive izcal-run
# session (a scheduled task does that), because its children load
# user32 (clipboard) and WinRT (PasswordVault) there. It also re-tests
# Credential Manager under an interactive logon, which the ssh runs
# could not exercise fairly.
#   izr.ps1  (no arguments; results go to $Dir\izr-run.log, end marker
#   line "izr done")
# Positions, as in izq: P0 the planter's own container while alive (the
# gate), P1 a sibling container, P2 the sibling after the planter ends,
# P3 the next holder of the recycled SID (delete + recreate the same
# name), P4 the plain user (a helper run by this driver). The clipboard
# is session-wide, so every position sits in this one session.
param([string]$Dir = "$env:USERPROFILE\izq")
$ErrorActionPreference = 'Continue'
Set-Location $Dir
$Log = "$Dir\izr-run.log"
$src = @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Text;
public static class R {
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int CreateAppContainerProfile(string n, string d, string s, IntPtr c, uint k, out IntPtr sid);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int DeleteAppContainerProfile(string n);
  [DllImport("userenv.dll", CharSet=CharSet.Unicode)] static extern int GetAppContainerFolderPath(string sid, out IntPtr path);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool ConvertSidToStringSidW(IntPtr sid, out IntPtr s);
  [DllImport("advapi32.dll")] static extern bool EqualSid(IntPtr a, IntPtr b);
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool OpenProcessToken(IntPtr p, uint a, out IntPtr t);
  [DllImport("advapi32.dll", SetLastError=true)] static extern bool GetTokenInformation(IntPtr t, int c, IntPtr b, int n, out int r);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredReadW(string t, int k, int f, out IntPtr c);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredDeleteW(string t, int k, int f);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredEnumerateW(string f, int fl, out int n, out IntPtr p);
  [DllImport("advapi32.dll")] static extern void CredFree(IntPtr c);
  [DllImport("kernel32.dll", SetLastError=true)] static extern IntPtr OpenProcess(uint a, bool i, uint pid);
  [DllImport("kernel32.dll")] static extern uint GetCurrentProcessId();
  [DllImport("kernel32.dll")] static extern bool ProcessIdToSessionId(uint pid, out uint s);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool InitializeProcThreadAttributeList(IntPtr l, int n, int f, ref IntPtr size);
  [DllImport("kernel32.dll", SetLastError=true)] static extern bool UpdateProcThreadAttribute(IntPtr l, uint f, IntPtr attr, IntPtr val, IntPtr size, IntPtr prev, IntPtr ret);
  [DllImport("kernel32.dll", SetLastError=true, CharSet=CharSet.Unicode)] static extern bool CreateProcessW(string app, StringBuilder cmd, IntPtr pa, IntPtr ta, bool inh, uint flags, IntPtr env, string dir, IntPtr si, out PI pi);
  [DllImport("kernel32.dll")] static extern uint WaitForSingleObject(IntPtr h, uint ms);
  [DllImport("kernel32.dll")] static extern bool GetExitCodeProcess(IntPtr h, out uint c);
  [DllImport("kernel32.dll")] static extern bool TerminateProcess(IntPtr h, uint c);
  [DllImport("kernel32.dll")] public static extern bool CloseHandle(IntPtr h);
  [DllImport("ole32.dll")] static extern void CoTaskMemFree(IntPtr p);
  [DllImport("ntdll.dll")] static extern int NtOpenDirectoryObject(out IntPtr h, uint a, ref OA oa);
  [StructLayout(LayoutKind.Sequential)] public struct PI { public IntPtr hp, ht; public uint pid, tid; }
  [StructLayout(LayoutKind.Sequential)] struct US { public ushort Len, Max; public IntPtr Buf; }
  [StructLayout(LayoutKind.Sequential)] struct OA { public int Len; public IntPtr Root, Name; public uint Attr; public IntPtr Sd, Qos; }

  public static string Str(IntPtr sid) { IntPtr s; ConvertSidToStringSidW(sid, out s); return Marshal.PtrToStringUni(s); }
  public static IntPtr Create(string name) { IntPtr sid; int hr = CreateAppContainerProfile(name, name, name, IntPtr.Zero, 0, out sid); if (hr != 0) throw new Exception("create " + name + " hr=0x" + hr.ToString("x")); return sid; }
  public static int Delete(string name) { return DeleteAppContainerProfile(name); }
  public static string Folder(IntPtr sid) { IntPtr p; int hr = GetAppContainerFolderPath(Str(sid), out p); if (hr != 0) return null; string s = Marshal.PtrToStringUni(p); CoTaskMemFree(p); return s; }
  public static uint MySession() { uint s; ProcessIdToSessionId(GetCurrentProcessId(), out s); return s; }

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

  static int NtDir(string path, out IntPtr h) {
    var b = Marshal.StringToHGlobalUni(path); var us = new US { Len = (ushort)(path.Length * 2), Max = (ushort)(path.Length * 2), Buf = b };
    IntPtr pus = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(US))); Marshal.StructureToPtr(us, pus, false);
    var oa = new OA { Len = Marshal.SizeOf(typeof(OA)), Name = pus, Attr = 0x40 };
    return NtOpenDirectoryObject(out h, 0x1, ref oa);
  }
  public static string DirState(IntPtr sid) { IntPtr h; int st = NtDir(@"\Sessions\" + MySession() + @"\AppContainerNamedObjects\" + Str(sid), out h); if (st == 0) { CloseHandle(h); return "present"; } if (st == unchecked((int)0xC0000034)) return "gone"; return "status=0x" + st.ToString("x8"); }

  public static List<uint> InContainer(IntPtr sid) {
    var o = new List<uint>(); IntPtr buf = Marshal.AllocHGlobal(256);
    foreach (var p in Process.GetProcesses()) {
      IntPtr h = OpenProcess(0x1000, false, (uint)p.Id); if (h == IntPtr.Zero) continue;
      IntPtr t; if (OpenProcessToken(h, 8, out t)) { int r; if (GetTokenInformation(t, 31, buf, 256, out r)) { IntPtr s = Marshal.ReadIntPtr(buf); if (s != IntPtr.Zero && EqualSid(s, sid)) o.Add((uint)p.Id); } CloseHandle(t); }
      CloseHandle(h);
    }
    return o;
  }
  public static string CredGet(string t) { IntPtr c; if (CredReadW(t, 1, 0, out c)) { CredFree(c); return "found"; } return "ERROR(" + Marshal.GetLastWin32Error() + ")"; }
  public static string CredEnum(string f) { int n; IntPtr p; if (CredEnumerateW(f, 0, out n, out p)) { CredFree(p); return n.ToString(); } return "ERROR(" + Marshal.GetLastWin32Error() + ")"; }
  public static string CredDel(string t) { return CredDeleteW(t, 1, 0) ? "ok" : "ERROR(" + Marshal.GetLastWin32Error() + ")"; }
}
'@
Add-Type -TypeDefinition $src -Language CSharp
$PS1 = 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
function Log($s) { $s; Add-Content -Path $Log -Value $s -Encoding ASCII }
# one in-container powershell child: script <mode> <resfile> <resource>
function ChildPS($sid, $script, $mode, $res, $resource, $sec = 120) {
  $p = [R]::Launch($PS1, "-NoProfile -ExecutionPolicy Bypass -File `"$script`" $mode `"$res`" $resource", $sid)
  $c = [R]::Wait($p, [uint32]($sec * 1000))
  if ($c -lt 0) { Log "child TIMEOUT after ${sec}s: killed"; [R]::Kill($p) } else { [void][R]::CloseHandle($p.hp) }
  $c
}
function ReadRes($f) { $h = @{}; foreach ($l in (Get-Content $f -EA SilentlyContinue)) { $i = $l.IndexOf('='); if ($i -gt 0) { $h[$l.Substring(0, $i)] = $l.Substring($i + 1) } }; $h }
function ShowRes($tag, $h) { $s = "$tag :"; foreach ($k in @('clip_new', 'clip', 'vault_new', 'vault', 'cred_new', 'cred', 'credenum', 'cookie_new', 'cookie')) { if ($h[$k]) { $s += " $k=$($h[$k])" } }; Log $s }
function Checks($sid, $folder, $tag, $token, $guid) {
  $res = "$folder\r-$tag.txt"
  [void](ChildPS $sid "$folder\clip.ps1" check $res $token)
  $c = ReadRes $res
  $res2 = "$folder\v-$tag.txt"
  [void](ChildPS $sid "$folder\helper.ps1" check-vault $res2 "izr-vault-$guid")
  ShowRes $tag ($c + (ReadRes $res2))
}
function WaitClear($sid) {
  $t = [Diagnostics.Stopwatch]::StartNew()
  while ($t.Elapsed.TotalSeconds -lt 60) { if (([R]::InContainer($sid)).Count -eq 0 -and [R]::DirState($sid) -eq 'gone') { return }; Start-Sleep -Milliseconds 200 }
  Log ("NOT clear: procs=" + (([R]::InContainer($sid)) -join ',') + " dir=" + [R]::DirState($sid))
}

Remove-Item $Log -ErrorAction SilentlyContinue
Log ("izr $(Get-Date -Format s) pid=$PID user=$(whoami) session=$([R]::MySession()) build=$([Environment]::OSVersion.Version)")
$guid = [guid]::NewGuid().ToString('N').Substring(0, 10)
$nameA = "izcal.izr.$guid"; $nameB = "izcal.izr.b$guid"
$sidA = [R]::Create($nameA); $folderA = [R]::Folder($sidA)
$sidB = [R]::Create($nameB); $folderB = [R]::Folder($sidB)
Log ("A=$nameA sid=$([R]::Str($sidA))"); Log ("B sid=$([R]::Str($sidB)) folder=$folderB")
$sidOrig = [R]::Str($sidA)
foreach ($f in $folderA, $folderB) { Copy-Item "$Dir\clip.ps1" -Destination $f -Force; Copy-Item "$Dir\helper.ps1" -Destination $f -Force }

# plant: clipboard + PasswordVault + (re-test) Credential Manager, all in A
$resA = "$folderA\r-plant.txt"
[void](ChildPS $sidA "$folderA\clip.ps1" plant $resA $guid)
$resV = "$folderA\v-plant.txt"
[void](ChildPS $sidA "$folderA\helper.ps1" plant-vault $resV "izr-vault-$guid")
$resC = "$folderA\c-plant.txt"
[void]([R]::Launch($Dir + '\izq.exe', "plant `"$resC`" izr$guid x", $sidA) | Out-Null)
$t = [Diagnostics.Stopwatch]::StartNew()
while ($t.Elapsed.TotalSeconds -lt 60) { if ((Test-Path $resC) -and (Get-Content $resC -EA SilentlyContinue) -contains 'done=1') { break }; Start-Sleep -Milliseconds 300 }
ShowRes 'plant(A)' ((ReadRes $resA) + (ReadRes $resV) + (ReadRes $resC))

# P0 gate: A itself, still alive (either plantable channel proves the detector)
Checks $sidA $folderA 'P0alive' $guid $guid
$g0 = ReadRes "$folderA\r-P0alive.txt"
$g0v = ReadRes "$folderA\v-P0alive.txt"
$gate = ($g0.clip -eq 'found') -or ($g0v.vault -eq 'found')
Log ("GATE: " + $(if ($gate) { "pass (clip=$($g0.clip) vault=$($g0v.vault))" } else { 'FAIL' }))

if ($gate) {
  Checks $sidB $folderB 'P1' $guid $guid
  $res0b = "$folderA\r-recheck.txt"
  [void](ChildPS $sidA "$folderA\clip.ps1" check $res0b $guid)
  if (($g0.clip -eq 'found') -and ((ReadRes $res0b).clip -ne 'found')) { Log 'note: clipboard no longer readable in A before kill' }
  foreach ($q in [R]::InContainer($sidA)) { Log "orphan $q killed"; [R]::KillPid($q) }
  WaitClear $sidA
  Checks $sidB $folderB 'P2' $guid $guid
  [void][R]::Delete($nameA)
  WaitClear $sidA
  $sidA = [R]::Create($nameA); $folderA = [R]::Folder($sidA)
  Log ("recreate A: sid same=$([R]::Str($sidA) -eq $sidOrig); folder exists=$(Test-Path $folderA)")
  Copy-Item "$Dir\clip.ps1" -Destination $folderA -Force; Copy-Item "$Dir\helper.ps1" -Destination $folderA -Force
  Checks $sidA $folderA 'P3' $guid $guid
  # P4: the plain user in the same session (no container)
  $p4r = "$Dir\r-p4.txt"; $p4v = "$Dir\v-p4.txt"
  & $PS1 -NoProfile -ExecutionPolicy Bypass -File "$Dir\clip.ps1" check $p4r $guid | Out-Null
  & $PS1 -NoProfile -ExecutionPolicy Bypass -File "$Dir\helper.ps1" check-vault $p4v "izr-vault-$guid" | Out-Null
  ShowRes 'P4 plain user' ((ReadRes $p4r) + (ReadRes $p4v) + @{ cred = [R]::CredGet("izq-cred-izr$guid"); credenum = [R]::CredEnum('izq-*') })
} else {
  Log 'skip P1-P4 (gate failed)'
}

"-- cleanup"
$rmres = "$folderA\v-rm.txt"
[void](ChildPS $sidA "$folderA\helper.ps1" remove-vault $rmres "izr-vault-$guid")
Log ("vault remove in A: " + (ReadRes $rmres).vault_remove)
Log ("cred delete: " + [R]::CredDel("izq-cred-izr$guid") + "; now: " + [R]::CredGet("izq-cred-izr$guid"))
foreach ($s in @($sidA, $sidB)) { foreach ($q in [R]::InContainer($s)) { Log "orphan $q killed"; [R]::KillPid($q) } }
$dl = "del A hr=0x{0:x}; del B hr=0x{1:x}" -f [R]::Delete($nameA), [R]::Delete($nameB)
Log $dl
Log "clipboard: intentionally left (session-wide, transient); the token was izr-clip-$guid"
Log 'izr done'
