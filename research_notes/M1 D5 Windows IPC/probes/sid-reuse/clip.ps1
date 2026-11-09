# clip.ps1: the clipboard channel for the izr probe. Runs inside whatever
# container spawns it (interactive session only: user32 fails its DllMain
# in session 0) or as the plain user. Writes exactly one key=value line:
#   clip_new= / clip=
# plant: put CF_TEXT "izr-clip-<Token>" on the clipboard.
# check: read it back without destroying it.
param([string]$Mode, [string]$ResFile, [string]$Token)
$ErrorActionPreference = 'Stop'
function Emit($s) { Set-Content -Path $ResFile -Value $s -Encoding ASCII }
$src = @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class Clip {
  [DllImport("user32.dll", SetLastError=true)] public static extern bool OpenClipboard(IntPtr h);
  [DllImport("user32.dll")] public static extern bool EmptyClipboard();
  [DllImport("user32.dll")] public static extern bool CloseClipboard();
  [DllImport("user32.dll")] public static extern bool IsClipboardFormatAvailable(uint f);
  [DllImport("user32.dll")] public static extern IntPtr SetClipboardData(uint f, IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetClipboardData(uint f);
  [DllImport("user32.dll")] public static extern IntPtr GlobalLock(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GlobalUnlock(IntPtr h);
  [DllImport("kernel32.dll")] public static extern IntPtr GlobalAlloc(uint f, UIntPtr n);
  [DllImport("kernel32.dll")] public static extern IntPtr GlobalFree(IntPtr h);
  [DllImport("kernel32.dll")] public static extern uint GetLastError();
  public static string Plant(string token) {
    for (int i = 0; i < 50; i++) { if (OpenClipboard(IntPtr.Zero)) { break; } System.Threading.Thread.Sleep(100); if (i == 49) return "clip_new=DENIED(" + GetLastError() + ")"; }
    string r;
    try {
      var b = Encoding.ASCII.GetBytes("izr-clip-" + token + "\0");
      IntPtr g = GlobalAlloc(0x0002 | 0x2000, (UIntPtr)(b.Length)); // GMEM_MOVEABLE|GMEM_ZEROINIT
      if (g == IntPtr.Zero) { r = "clip_new=ERROR(" + GetLastError() + ")"; }
      else {
        IntPtr p = GlobalLock(g);
        Marshal.Copy(b, 0, p, b.Length);
        GlobalUnlock(g);
        r = SetClipboardData(1, g) != IntPtr.Zero ? "clip_new=ok" : "clip_new=DENIED(" + GetLastError() + ")"; // CF_TEXT
      }
    } finally { CloseClipboard(); }
    return r;
  }
  public static string Check(string token) {
    if (!IsClipboardFormatAvailable(1)) return "clip=gone";
    for (int i = 0; i < 50; i++) { if (OpenClipboard(IntPtr.Zero)) { break; } System.Threading.Thread.Sleep(100); if (i == 49) return "clip=DENIED(" + GetLastError() + ")"; }
    string r;
    try {
      IntPtr h = GetClipboardData(1);
      if (h == IntPtr.Zero) { r = "clip=gone"; }
      else {
        IntPtr p = GlobalLock(h);
        int n = 0; while (Marshal.ReadByte(p, n) != 0 && n < 4096) n++;
        var b = new byte[n]; Marshal.Copy(p, b, 0, n);
        GlobalUnlock(h);
        string s = Encoding.ASCII.GetString(b);
        r = s.Contains("izr-clip-" + token) ? "clip=found" : "clip=other";
      }
    } finally { CloseClipboard(); }
    return r;
  }
}
'@
try {
  Add-Type -TypeDefinition $src -Language CSharp
  if ($Mode -eq 'plant') { Emit ([Clip]::Plant($Token)) } else { Emit ([Clip]::Check($Token)) }
} catch {
  $m = $_.Exception.Message; if ($m.Length -gt 120) { $m = $m.Substring(0, 120) }
  $k = 'clip'; if ($Mode -eq 'plant') { $k = 'clip_new' }
  Emit ("{0}=ERROR {1}" -f $k, $m)
}
