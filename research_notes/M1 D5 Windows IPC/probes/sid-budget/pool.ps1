# Top kernel pool tags by bytes (SystemPoolTagInformation = 22).
param([int]$Top = 25, [string]$Dump = "")
$src = @'
using System; using System.Collections.Generic; using System.Runtime.InteropServices; using System.Text;
public static class Pool {
  [DllImport("ntdll.dll")] static extern int NtQuerySystemInformation(int cls, IntPtr buf, int len, out int ret);
  public static string Run(int top, string dump) {
    int len = 1 << 20, ret; IntPtr b = IntPtr.Zero; int st;
    while (true) { b = Marshal.AllocHGlobal(len); st = NtQuerySystemInformation(22, b, len, out ret); if (st != unchecked((int)0xC0000004)) break; Marshal.FreeHGlobal(b); len *= 2; }
    if (st != 0) return "status=0x" + st.ToString("x");
    int n = Marshal.ReadInt32(b, 0); var rows = new List<Tuple<long, string>>(); long pagedT = 0, nonT = 0;
    // SYSTEM_POOLTAG (x64): Tag[4], PagedAllocs u32, PagedFrees u32, pad, PagedUsed u64, NonPagedAllocs u32, NonPagedFrees u32, NonPagedUsed u64 = 40 bytes; array at offset 8
    for (int i = 0; i < n; i++) {
      IntPtr e = b + 8 + 40 * i; var t = new byte[4]; Marshal.Copy(e, t, 0, 4);
      long pa = (uint)Marshal.ReadInt32(e, 4), pf = (uint)Marshal.ReadInt32(e, 8), pu = Marshal.ReadInt64(e, 16);
      long na = (uint)Marshal.ReadInt32(e, 24), nf = (uint)Marshal.ReadInt32(e, 28), nu = Marshal.ReadInt64(e, 32);
      pagedT += pu; nonT += nu;
      string tag = Encoding.ASCII.GetString(t).Replace("\0", " ");
      rows.Add(Tuple.Create(pu + nu, string.Format("{0}  paged={1,12:N0} (live {2,9:N0})  nonpaged={3,12:N0} (live {4,9:N0})", tag, pu, pa - pf, nu, na - nf)));
    }
    if (dump != "") { var sb = new StringBuilder(); for (int i = 0; i < n; i++) { IntPtr e = b + 8 + 40 * i; var t = new byte[4]; Marshal.Copy(e, t, 0, 4); sb.AppendLine(Encoding.ASCII.GetString(t).Replace("\0"," ") + "\t" + ((uint)Marshal.ReadInt32(e,4) - (long)(uint)Marshal.ReadInt32(e,8)) + "\t" + Marshal.ReadInt64(e,16) + "\t" + ((uint)Marshal.ReadInt32(e,24) - (long)(uint)Marshal.ReadInt32(e,28)) + "\t" + Marshal.ReadInt64(e,32)); } System.IO.File.WriteAllText(dump, sb.ToString()); }
    rows.Sort((x, y) => y.Item1.CompareTo(x.Item1));
    var o = new StringBuilder(); o.AppendLine("tags=" + n + " pagedTotal=" + pagedT.ToString("N0") + " nonpagedTotal=" + nonT.ToString("N0"));
    for (int i = 0; i < top && i < rows.Count; i++) o.AppendLine(rows[i].Item2);
    return o.ToString();
  }
}
'@
Add-Type -TypeDefinition $src -Language CSharp
[Pool]::Run($Top, $Dump)
