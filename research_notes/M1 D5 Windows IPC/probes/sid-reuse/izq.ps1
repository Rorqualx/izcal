# izq.ps1: the cross-container leakage probe. Does anything a container's
# planter leaves reach (P1) a sibling container, (P4) the plain user, or
# (P3) the next holder of the recycled SID? Runs as a standard user;
# children are Rust (izq.exe); WinRT goes through helper.ps1 in the container.
#   izq.ps1 -Mode smoke   bring-up: one container, plant+check, driver spots
#   izq.ps1 -Mode run     the full five-position run
param([string]$Mode = 'run')
$Dir = "$env:USERPROFILE\izq"
$Exe = "$Dir\izq.exe"
$HelperSrc = "$Dir\helper.ps1"
$src = @'
using System; using System.Collections.Generic; using System.Diagnostics; using System.Runtime.InteropServices; using System.Text;
public static class Q {
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
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredReadW(string t, int k, int f, out IntPtr p);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredEnumerateW(string f, int fl, out int n, out IntPtr a);
  [DllImport("advapi32.dll", CharSet=CharSet.Unicode)] static extern bool CredDeleteW(string t, int k, int f);
  [DllImport("advapi32.dll")] static extern void CredFree(IntPtr b);
  [DllImport("wininet.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern bool InternetGetCookieW(string u, string n, StringBuilder d, ref int l);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern ushort GlobalFindAtomW(string n);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern ushort GlobalDeleteAtomW(ushort a);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern IntPtr OpenEventW(uint a, bool i, string n);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode)] static extern IntPtr OpenFileMappingW(uint a, bool i, string n);
  [DllImport("kernel32.dll", CharSet=CharSet.Unicode, SetLastError=true)] static extern IntPtr CreateFileW(string n, uint a, uint s, IntPtr sa, uint c, uint f, IntPtr t);
  [DllImport("kernel32.dll")] static extern IntPtr GlobalLock(IntPtr h);
  [DllImport("kernel32.dll")] static extern bool GlobalUnlock(IntPtr h);
  [DllImport("kernel32.dll")] static extern bool OpenClipboard(IntPtr h);
  [DllImport("kernel32.dll")] static extern bool CloseClipboard();
  [DllImport("kernel32.dll")] static extern bool IsClipboardFormatAvailable(uint f);
  [DllImport("kernel32.dll")] static extern IntPtr GetClipboardData(uint f);
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

  static int NtDir(string path, out IntPtr h) {
    var b = Marshal.StringToHGlobalUni(path); var us = new US { Len = (ushort)(path.Length * 2), Max = (ushort)(path.Length * 2), Buf = b };
    IntPtr pus = Marshal.AllocHGlobal(Marshal.SizeOf(typeof(US))); Marshal.StructureToPtr(us, pus, false);
    var oa = new OA { Len = Marshal.SizeOf(typeof(OA)), Name = pus, Attr = 0x40 };
    return NtOpenDirectoryObject(out h, 0x1, ref oa);
  }
  public static string NamedDir(IntPtr sid) { uint s; ProcessIdToSessionId(GetCurrentProcessId(), out s); return @"\Sessions\" + s + @"\AppContainerNamedObjects\" + Str(sid); }
  public static string DirState(IntPtr sid) { IntPtr h; int st = NtDir(NamedDir(sid), out h); if (st == 0) { CloseHandle(h); return "present"; } if (st == unchecked((int)0xC0000034)) return "gone"; return "status=0x" + st.ToString("x8"); }

  // Processes of this user whose token is in the container `sid` (conhost included).
  public static List<uint> InContainer(IntPtr sid) {
    var o = new List<uint>(); IntPtr buf = Marshal.AllocHGlobal(256);
    foreach (var p in Process.GetProcesses()) {
      IntPtr h = OpenProcess(0x1000, false, (uint)p.Id); if (h == IntPtr.Zero) continue;
      IntPtr t; if (OpenProcessToken(h, 8, out t)) { int r; if (GetTokenInformation(t, 31, buf, 256, out r)) { IntPtr s = Marshal.ReadIntPtr(buf); if (s != IntPtr.Zero && EqualSid(s, sid)) o.Add((uint)p.Id); } CloseHandle(t); }
      CloseHandle(h);
    }
    return o;
  }

  // ---- plain-user position checks ----
  public static string CredGet(string target) { IntPtr p; if (CredReadW(target, 1, 0, out p)) { CredFree(p); return "found"; } int e = Marshal.GetLastWin32Error(); return e == 1168 ? "gone" : "ERROR(" + e + ")"; }
  public static string CredEnum(string filter) { int n; IntPtr a; if (CredEnumerateW(filter, 0, out n, out a)) { CredFree(a); return n.ToString(); } int e = Marshal.GetLastWin32Error(); return e == 1168 ? "0" : "ERROR(" + e + ")"; }
  public static string CredDel(string target) { return CredDeleteW(target, 1, 0) ? "ok" : "ERROR(" + Marshal.GetLastWin32Error() + ")"; }
  public static string CookieGet(string url) { var sb = new StringBuilder(4096); int l = 4096; if (InternetGetCookieW(url, null, sb, ref l)) return sb.ToString(); int e = Marshal.GetLastWin32Error(); return e == 259 ? "gone" : "DENIED(" + e + ")"; }
  public static int AtomFind(string name) { return GlobalFindAtomW(name); }
  public static string AtomDel(int idx) { return GlobalDeleteAtomW((ushort)idx) == 0 ? "ok" : "ERROR"; }
  public static string GOpen(string kind, string name) {
    IntPtr h;
    if (kind == "ev") h = OpenEventW(1, false, @"Global\" + name);
    else if (kind == "map") h = OpenFileMappingW(4, false, @"Global\" + name);
    else h = CreateFileW(@"\\.\pipe\" + name, 0xC0000000, 0, IntPtr.Zero, 3, 0, IntPtr.Zero);
    if (kind == "pipe" ? h != (IntPtr)(-1) : h != IntPtr.Zero) { CloseHandle(h); return "found"; }
    int e = kind == "pipe" ? Marshal.GetLastWin32Error() : Marshal.GetLastWin32Error();
    return e == 2 ? "gone" : "DENIED(" + e + ")";
  }
  public static string ClipGet() {
    for (int i = 0; i < 25; i++) { if (OpenClipboard(IntPtr.Zero)) { try { if (!IsClipboardFormatAvailable(1)) return "gone"; IntPtr h = GetClipboardData(1); if (h == IntPtr.Zero) return "gone"; IntPtr p = GlobalLock(h); if (p == IntPtr.Zero) return "gone"; string s = Marshal.PtrToStringAnsi(p); GlobalUnlock(h); return string.IsNullOrEmpty(s) ? "gone" : s; } finally { CloseClipboard(); } } System.Threading.Thread.Sleep(200); }
    return "ERROR(open)";
  }

  // ---- ACL scans (unelevated; unreadable = recorded, never elevated) ----
  static Microsoft.Win32.RegistryKey Base(string r) { return r == "HKLM" ? Microsoft.Win32.Registry.LocalMachine : r == "HKCR" ? Microsoft.Win32.Registry.ClassesRoot : Microsoft.Win32.Registry.CurrentUser; }
  public static string[] SubKeys(string full) {
    var i = full.IndexOf('\\'); Microsoft.Win32.RegistryKey k;
    try { if (i < 0) k = Base(full); else k = Base(full.Substring(0, i)).OpenSubKey(full.Substring(i + 1), false); } catch { return new string[0]; }
    if (k == null) return new string[0];
    try { return k.GetSubKeyNames(); } catch { return new string[0]; } finally { k.Close(); }
  }
  static bool WritableFs(int rights) { return (rights & 0x216) != 0 || (rights & 0x50000000) != 0 || rights == 0x30101; }
  static bool WritableReg(int rights) { return (rights & 0x20006) != 0 || (rights & 0x50000000) != 0; }
  static bool Shared(string id) { return id == "S-1-15-2-1" || id == "S-1-15-2-2"; }
  public static string ScanFs(string[] paths, string sid, string dump) {
    int unread = 0, n = 0; var hits = new List<string>(); var seenF = new HashSet<string>();
    foreach (var p in paths) {
      System.Security.AccessControl.FileSystemSecurity sec; bool isDir;
      try { if (System.IO.Directory.Exists(p)) { sec = System.IO.Directory.GetAccessControl(p); isDir = true; }
            else if (System.IO.File.Exists(p)) { sec = System.IO.File.GetAccessControl(p); isDir = false; }
            else continue; }
      catch { unread++; continue; }
      n++;
      foreach (System.Security.AccessControl.FileSystemAccessRule r in sec.GetAccessRules(true, true, typeof(System.Security.Principal.SecurityIdentifier))) {
        var id = r.IdentityReference.Value; var rights = (int)r.FileSystemRights;
        if (id == sid) hits.Add(p);
        if (r.AccessControlType == System.Security.AccessControl.AccessControlType.Allow && Shared(id) && WritableFs(rights) && seenF.Add(p))
          System.IO.File.AppendAllText(dump, (isDir ? "dir\t" : "file\t") + p + "\n");
      }
    }
    return "checked=" + n + " unread=" + unread + " sidHits=[" + string.Join(";", hits) + "]";
  }
  public static string ScanReg(string[] keys, string sid, string dump) {
    int unread = 0, n = 0; var hits = new List<string>(); var seen = new HashSet<string>();
    foreach (var full in keys) {
      var i = full.IndexOf('\\'); Microsoft.Win32.RegistryKey k;
      try { if (i < 0) k = Base(full); else k = Base(full.Substring(0, i)).OpenSubKey(full.Substring(i + 1), false); } catch { unread++; continue; }
      if (k == null) continue;
      try {
        n++;
        var sec = k.GetAccessControl(System.Security.AccessControl.AccessControlSections.Access);
        foreach (System.Security.AccessControl.RegistryAccessRule r in sec.GetAccessRules(true, true, typeof(System.Security.Principal.SecurityIdentifier))) {
          var id = r.IdentityReference.Value; var rights = (int)r.RegistryRights;
          if (id == sid) hits.Add(k.Name);
          if (r.AccessControlType == System.Security.AccessControl.AccessControlType.Allow && Shared(id) && WritableReg(rights) && seen.Add(k.Name))
            System.IO.File.AppendAllText(dump, "reg\t" + full + "\n");
        }
      } catch { unread++; } finally { try { k.Close(); } catch {} }
    }
    return "checked=" + n + " unread=" + unread + " sidHits=[" + string.Join(";", hits) + "]";
  }
}
'@
Add-Type -TypeDefinition $src -Language CSharp

function ReadRes($p) {
  $h = @{}
  if (Test-Path $p) { foreach ($l in Get-Content $p) { $i = $l.IndexOf('='); if ($i -gt 0) { $h[$l.Substring(0, $i)] = $l.Substring($i + 1) } } }
  else { $h['MISSING'] = '1' }
  $h
}
function ShowRes($tag, $h) {
  $order = @('cred','credenum','vault','cookie','atom','clip','gev','gmap','gpipe','wo_read','cred_new','cookie_new','atom_new','gev_new','gmap_new','gpipe_new')
  $parts = @(); foreach ($k in $order) { if ($h.ContainsKey($k)) { $parts += "$k=$($h[$k])" } }
  foreach ($k in ($h.Keys | Sort-Object)) { if ($k -like 'wo*' -and $order -notcontains $k) { $parts += "$k=$($h[$k])" } }
  "== $tag : " + ($parts -join ' ')
}
function Child($sid, $argline, $sec) {
  $p = [Q]::Launch($Exe, $argline, $sid)
  $c = [Q]::Wait($p, [uint32]($sec * 1000))
  if ($c -lt 0) { "child TIMEOUT after ${sec}s: killed"; [Q]::Kill($p) } else { [void][Q]::CloseHandle($p.hp) }
  $c
}
function CopyIn($folder, $f, $name) { try { $dst = if ($name) { Join-Path $folder $name } else { $folder }; Copy-Item $f -Destination $dst -Force -EA Stop; $true } catch { "FAIL: " + $_.Exception.Message.Substring(0, [Math]::Min(60, $_.Exception.Message.Length)) } }
function WaitClear($sid) {
  $t = [Diagnostics.Stopwatch]::StartNew()
  while ($t.Elapsed.TotalSeconds -lt 60) { if (([Q]::InContainer($sid)).Count -eq 0 -and [Q]::DirState($sid) -eq 'gone') { return "clear after {0:N1} s" -f $t.Elapsed.TotalSeconds }; Start-Sleep -Milliseconds 200 }
  "NOT clear: procs=" + (([Q]::InContainer($sid)) -join ',') + " dir=" + [Q]::DirState($sid)
}
function Cleanup($guid, $nameA, $nameB, $sidA, $sidB, $atomIdx) {
  "-- cleanup"
  "cred delete: " + [Q]::CredDel("izq-cred-$guid") + "; now: " + [Q]::CredGet("izq-cred-$guid")
  if ($atomIdx -gt 0) { "atom delete: " + [Q]::AtomDel($atomIdx) }
  foreach ($s in @($sidA, $sidB)) { foreach ($q in [Q]::InContainer($s)) { "orphan $q (" + $(try { (Get-Process -Id $q).ProcessName } catch { '?' }) + "): killed"; [Q]::KillPid($q) } }
  "del A hr=0x{0:x}; del B hr=0x{1:x}" -f [Q]::Delete($nameA), [Q]::Delete($nameB)
  "A dir: " + [Q]::DirState($sidA) + "; B dir: " + [Q]::DirState($sidB)
  "intentional leftovers: the cookie on izq-$guid.probe (the probe only reads it back); the clipboard and PasswordVault channels were not probed (user32 fails DllMain inside the container in this session)"
}
function P4Checks($guid) {
  $P4 = @{}
  $P4.cred = [Q]::CredGet("izq-cred-$guid")
  $P4.credenum = [Q]::CredEnum("izq-*")
  $ck = [Q]::CookieGet("https://izq-$guid.probe/")
  $P4.cookie = if ($ck -eq 'gone' -or $ck -like 'DENIED*' -or $ck -like 'ERROR*') { $ck } elseif ($ck.Contains($guid)) { 'found' } else { 'other' }
  $P4.atom = if ([Q]::AtomFind("izq-atom-$guid") -ne 0) { 'found' } else { 'gone' }
  $P4.gev = [Q]::GOpen('ev', "izq-g-$guid")
  $P4.gmap = [Q]::GOpen('map', "izq-m-$guid")
  $P4.gpipe = [Q]::GOpen('pipe', "izq-p-$guid")
  $P4
}

"izq $Mode $(Get-Date -Format s) pid=$PID user=$(whoami) build=$([Environment]::OSVersion.Version)"

if ($Mode -eq 'smoke') {
  $g = [guid]::NewGuid().ToString('N').Substring(0, 8)
  $n = "izcal.izq.smoke.$g"; $guid = "smoke$g"
  "profile $n"
  $sid = [Q]::Create($n); $folder = [Q]::Folder($sid); "folder $folder"
  $w = try { Set-Content -Path (Join-Path $folder 't.txt') -Value 'x' -EA Stop; 'ok' } catch { "FAIL: " + $_.Exception.Message.Substring(0, [Math]::Min(80, $_.Exception.Message.Length)) }
  "driver write into container folder: $w"
  "helper into folder: " + (CopyIn $folder $HelperSrc)
  $res = Join-Path $folder 'res.txt'
  $c = Child $sid "plant `"$res`" $guid `"$folder\helper.ps1`" 3" 240
  "plant exit=$c"; ShowRes 'plant' (ReadRes $res)
  $res2 = Join-Path $folder 'res2.txt'
  $c = Child $sid "check `"$res2`" $guid `"$folder\helper.ps1`"" 240
  "check exit=$c"; ShowRes 'check' (ReadRes $res2)
  "driver: cred=" + [Q]::CredGet("izq-cred-$guid") + " credenum=" + [Q]::CredEnum("izq-*") + " atom=" + [Q]::AtomFind("izq-atom-$guid") + " cookie=" + [Q]::CookieGet("https://izq-$guid.probe/") + " gev=" + [Q]::GOpen('ev', "izq-g-$guid")
  foreach ($q in [Q]::InContainer($sid)) { "orphan ${q}: killed"; [Q]::KillPid($q) }
  [void][Q]::Delete($n); "deleted; dir " + [Q]::DirState($sid)
  "cleanup note: the smoke cookie is intentionally left"
  return
}

# ---------- full run ----------
$guid = [guid]::NewGuid().ToString('N').Substring(0, 10)
$nameA = "izcal.izq.$guid"; $nameB = "izcal.izq.b$guid"

$fsPaths = @( 'C:\' ) +
  @(Get-ChildItem 'C:\' -Force -EA SilentlyContinue | ForEach-Object FullName) +
  @(Get-ChildItem 'C:\ProgramData' -Force -EA SilentlyContinue | ForEach-Object FullName) +
  @("$env:LOCALAPPDATA\Packages") +
  @(Get-ChildItem "$env:LOCALAPPDATA\Packages" -Force -EA SilentlyContinue | ForEach-Object FullName)
$regKeys = @('HKCU') + @([Q]::SubKeys('HKCU')) +
  @('HKCU\Software\Classes\Local Settings') + @([Q]::SubKeys('HKCU\Software\Classes\Local Settings')) +
  @('HKLM') + @([Q]::SubKeys('HKLM')) + @([Q]::SubKeys('HKLM\SOFTWARE'))
"bounds: fs=$($fsPaths.Count) paths (C:\ + top level, ProgramData +1, Packages +1), reg=$($regKeys.Count) keys (HKCU +1, Local Settings +1, HKLM top + HKLM\SOFTWARE +1)"

function Snap($tag, $sidStr) {
  $dump = "$Dir\shared-$tag.tsv"
  Set-Content -Path $dump -Value '' -Encoding ASCII
  "snapshot $tag fs : " + [Q]::ScanFs($fsPaths, $sidStr, $dump)
  "snapshot $tag reg: " + [Q]::ScanReg($regKeys, $sidStr, $dump)
}

Snap 0 ''
$sidA = [Q]::Create($nameA); $folderA = [Q]::Folder($sidA)
$sidB = [Q]::Create($nameB); $folderB = [Q]::Folder($sidB)
"A=$nameA sid=$([Q]::Str($sidA))"
"B sid=$([Q]::Str($sidB)) folder=$folderB"
Snap 1 ([Q]::Str($sidA))
"helper into A: " + (CopyIn $folderA $HelperSrc) + "; into B: " + (CopyIn $folderB $HelperSrc)
$helperA = "$folderA\helper.ps1"; $helperB = "$folderB\helper.ps1"
$sharedInA = "$folderA\shared.tsv"
CopyIn $folderA "$Dir\shared-1.tsv" 'shared.tsv' | Out-Null

# plant (stays alive until killed)
$resPlant = "$folderA\res-plant.txt"
$planter = [Q]::Launch($Exe, "plant `"$resPlant`" $guid `"$helperA`" 900", $sidA)
$t = [Diagnostics.Stopwatch]::StartNew()
while ($t.Elapsed.TotalSeconds -lt 180) { if ((Test-Path $resPlant) -and (Get-Content $resPlant -EA SilentlyContinue) -contains 'done=1') { break }; Start-Sleep -Milliseconds 500 }
$plantRes = ReadRes $resPlant
if (-not $plantRes.done) { "plant: NO done marker after 180 s" }
ShowRes 'plant (A)' $plantRes
$atomIdx = 0; if ($plantRes.atom_new) { [void][int]::TryParse($plantRes.atom_new, [ref]$atomIdx) }

# P0 control: container A itself, planter alive
$res0 = "$folderA\res-p0.txt"
[void](Child $sidA "check `"$res0`" $guid `"$helperA`" `"$sharedInA`"" 300)
$P0 = ReadRes $res0
ShowRes 'P0 A-alive (gate)' $P0
$gate = $true; $n = 0
foreach ($c in @(@('gev_new', 'gev'), @('gmap_new', 'gmap'))) {
  if ($plantRes[$c[0]] -eq 'ok') { $n++; $gate = $gate -and ($P0[$c[1]] -eq 'found') }
}
$gate = $gate -and ($n -ge 1)
"GATE: " + $(if ($gate) { "pass ($n plantable channels)" } else { "FAIL (plantable=$n)" })

if ($gate) {
  # P1 sibling B while A's planter is alive
  $res1 = "$folderB\res-p1.txt"
  [void](Child $sidB "check `"$res1`" $guid `"$helperB`"" 300)
  $P1 = ReadRes $res1
  ShowRes 'P1 B, A alive' $P1

  # end the planter; wait until A is empty and its named-object dir is gone
  [Q]::Kill($planter)
  "A after kill: " + (WaitClear $sidA)

  # P2 sibling B after A's planter exited (A profile still present)
  $res2 = "$folderB\res-p2.txt"
  [void](Child $sidB "check `"$res2`" $guid `"$helperB`"" 300)
  $P2 = ReadRes $res2
  ShowRes 'P2 B, A exited' $P2

  # P3 recycle A: same name, same SID, fresh profile
  "del A hr=0x{0:x}" -f [Q]::Delete($nameA)
  Snap 2 ([Q]::Str($sidA))
  $sidA2 = [Q]::Create($nameA)
  $sidSame = ([Q]::Str($sidA2) -eq [Q]::Str($sidA))
  $sidA = $sidA2; $folderA = [Q]::Folder($sidA)
  "recreate A: sid same=$sidSame"
  CopyIn $folderA $HelperSrc | Out-Null; CopyIn $folderA "$Dir\shared-1.tsv" 'shared.tsv' | Out-Null
  $res3 = "$folderA\res-p3.txt"
  [void](Child $sidA "check `"$res3`" $guid `"$folderA\helper.ps1`" `"$folderA\shared.tsv`"" 300)
  $P3 = ReadRes $res3
  ShowRes 'P3 recycled holder' $P3
  Snap 3 ([Q]::Str($sidA))

  # P4 plain user (this driver, same session)
  $P4 = P4Checks $guid
  ShowRes 'P4 plain user' $P4

  "-- summary (plant-time results in brackets)"
  $pl = @{ cred = $plantRes.cred_new; cookie = $plantRes.cookie_new; atom = $plantRes.atom_new; gev = $plantRes.gev_new; gmap = $plantRes.gmap_new; gpipe = $plantRes.gpipe_new }
  $channels = 'cred','credenum','cookie','atom','gev','gmap','gpipe'
  $V = { param($h, $k) $(if ($h -and $h.ContainsKey($k)) { $h[$k] } else { '-' }) }
  foreach ($ch in $channels) {
    "{0,-9} plant=[{1}] P0={2} P1={3} P2={4} P3={5} P4={6}" -f $ch, (& $V $pl $ch), (& $V $P0 $ch), (& $V $P1 $ch), (& $V $P2 $ch), (& $V $P3 $ch), (& $V $P4 $ch)
  }
  foreach ($pos in @(@('P0', $P0), @('P3', $P3))) {
    $h = $pos[1]; $wo = @()
    foreach ($k in ($h.Keys | Sort-Object)) { if ($k -like 'wo*') { $wo += "$k=$($h[$k])" } }
    "wo opens $($pos[0]): " + $(if ($wo) { $wo -join ' ' } else { '(none recorded)' })
  }
  Cleanup $guid $nameA $nameB $sidA $sidB $atomIdx
} else {
  "GATE FAILED - stopping before P1..P4"
  [Q]::Kill($planter)
  Cleanup $guid $nameA $nameB $sidA $sidB $atomIdx
}
