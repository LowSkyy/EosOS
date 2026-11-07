# Lettura e impostazione della scala (DPI) del monitor primario, senza passare dalle Impostazioni.
# Usa le chiamate di configurazione dello schermo di Windows (DisplayConfig). Quella di
# impostazione (-4) non e' documentata ma e' quella che usano le Impostazioni stesse: la scala
# cambia subito, senza disconnettersi, e le app ricevono WM_DPICHANGED.
#
# Si include con:  . "$PSScriptRoot\lib-dpi.ps1"
#   Get-PrimaryScale            -> scala corrente del monitor primario in percentuale (100, 125...)
#   Get-PrimaryScaleRange       -> scale supportate dal monitor (elenco di percentuali)
#   Set-PrimaryScale 150        -> imposta il 150% (solo se supportato); restituisce la scala ottenuta
#
# ATTENZIONE: cambiare la scala ridisegna tutte le finestre aperte. Chi usa questa libreria deve
# ripristinare la scala originale in un blocco finally.
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class DS {
  [StructLayout(LayoutKind.Sequential)] public struct LUID { public uint Low; public int High; }
  [StructLayout(LayoutKind.Sequential)] public struct HDR { public uint Type; public uint Size; public LUID Adapter; public uint Id; }
  [StructLayout(LayoutKind.Sequential)] public struct GETDPI { public HDR H; public int Min; public int Cur; public int Max; }
  [StructLayout(LayoutKind.Sequential)] public struct SETDPI { public HDR H; public int Rel; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct SRCNAME { public HDR H; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string Name; }
  [DllImport("user32.dll")] static extern int GetDisplayConfigBufferSizes(uint flags, out uint paths, out uint modes);
  [DllImport("user32.dll")] static extern int QueryDisplayConfig(uint flags, ref uint paths, IntPtr pathBuf, ref uint modes, IntPtr modeBuf, IntPtr topo);
  [DllImport("user32.dll", EntryPoint="DisplayConfigGetDeviceInfo")] static extern int GetInfo(ref GETDPI p);
  [DllImport("user32.dll", EntryPoint="DisplayConfigGetDeviceInfo", CharSet=CharSet.Unicode)] static extern int GetName(ref SRCNAME p);
  [DllImport("user32.dll", EntryPoint="DisplayConfigSetDeviceInfo")] static extern int SetInfo(ref SETDPI p);
  [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr h, int t, out uint x, out uint y);
  [DllImport("user32.dll")] static extern IntPtr MonitorFromPoint(int x, int y, uint flags);
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  public static void Aware() { SetProcessDpiAwarenessContext(new IntPtr(-4)); }

  // Scale che Windows propone, in ordine: indice = scala relativa + indice della raccomandata.
  public static readonly int[] Steps = { 100, 125, 150, 175, 200, 225, 250, 300, 350, 400, 450, 500 };

  // Trova adattatore e id della sorgente il cui nome GDI (\\.\DISPLAYn) e' `gdi`.
  static bool Find(string gdi, out LUID adapter, out uint id) {
    adapter = new LUID(); id = 0;
    uint np, nm;
    if (GetDisplayConfigBufferSizes(2, out np, out nm) != 0) return false;
    IntPtr pb = Marshal.AllocHGlobal((int)np * 72), mb = Marshal.AllocHGlobal((int)nm * 64);
    try {
      if (QueryDisplayConfig(2, ref np, pb, ref nm, mb, IntPtr.Zero) != 0) return false;
      for (int i = 0; i < np; i++) {
        var a = new LUID { Low = (uint)Marshal.ReadInt32(pb, i * 72), High = Marshal.ReadInt32(pb, i * 72 + 4) };
        uint sid = (uint)Marshal.ReadInt32(pb, i * 72 + 8);
        var n = new SRCNAME(); n.H.Type = 1; n.H.Size = (uint)Marshal.SizeOf(typeof(SRCNAME)); n.H.Adapter = a; n.H.Id = sid;
        if (GetName(ref n) == 0 && n.Name == gdi) { adapter = a; id = sid; return true; }
      }
    } finally { Marshal.FreeHGlobal(pb); Marshal.FreeHGlobal(mb); }
    return false;
  }

  // Scala corrente (percentuale) del monitor che contiene il punto (0,0), cioe' il primario.
  public static int CurrentPercent() {
    uint x, y; GetDpiForMonitor(MonitorFromPoint(0, 0, 1), 0, out x, out y); return (int)(x * 100 / 96);
  }

  // Min/Cur/Max relativi alla scala raccomandata, o null se non disponibili.
  public static int[] Rel(string gdi) {
    LUID a; uint id; if (!Find(gdi, out a, out id)) return null;
    var g = new GETDPI(); g.H.Type = unchecked((uint)-3); g.H.Size = (uint)Marshal.SizeOf(typeof(GETDPI)); g.H.Adapter = a; g.H.Id = id;
    if (GetInfo(ref g) != 0) return null;
    return new[] { g.Min, g.Cur, g.Max };
  }

  public static bool SetRel(string gdi, int rel) {
    LUID a; uint id; if (!Find(gdi, out a, out id)) return false;
    var s = new SETDPI(); s.H.Type = unchecked((uint)-4); s.H.Size = (uint)Marshal.SizeOf(typeof(SETDPI)); s.H.Adapter = a; s.H.Id = id; s.Rel = rel;
    return SetInfo(ref s) == 0;
  }
}
'@
[DS]::Aware()

function Get-PrimaryGdi { [System.Windows.Forms.Screen]::PrimaryScreen.DeviceName }

function Get-PrimaryScale { [DS]::CurrentPercent() }

# Scale supportate dal monitor primario (percentuali), o $null se la configurazione non e' leggibile.
function Get-PrimaryScaleRange {
  $rel = [DS]::Rel((Get-PrimaryGdi)); if (-not $rel) { return $null }
  $cur = [DS]::CurrentPercent(); $curIdx = [Array]::IndexOf([DS]::Steps, $cur)
  if ($curIdx -lt 0) { return $null }
  $rec = $curIdx - $rel[1]                       # indice della scala raccomandata
  ($rel[0]..$rel[2]) | ForEach-Object { $i = $rec + $_; if ($i -ge 0 -and $i -lt [DS]::Steps.Length) { [DS]::Steps[$i] } }
}

# Imposta la scala del monitor primario. Restituisce la scala effettiva dopo il cambio.
function Set-PrimaryScale([int]$Percent) {
  $gdi = Get-PrimaryGdi
  $rel = [DS]::Rel($gdi); if (-not $rel) { throw 'configurazione dello schermo non leggibile' }
  $cur = [DS]::CurrentPercent(); $curIdx = [Array]::IndexOf([DS]::Steps, $cur)
  $tgtIdx = [Array]::IndexOf([DS]::Steps, $Percent)
  if ($curIdx -lt 0 -or $tgtIdx -lt 0) { throw "scala non gestita ($cur% -> $Percent%)" }
  $rec = $curIdx - $rel[1]
  $want = $tgtIdx - $rec
  if ($want -lt $rel[0] -or $want -gt $rel[2]) { throw "il monitor non supporta il $Percent%" }
  if ($want -ne $rel[1]) {
    if (-not [DS]::SetRel($gdi, $want)) { throw "impostazione del $Percent% rifiutata" }
    $t0 = Get-Date
    while ([DS]::CurrentPercent() -ne $Percent -and ((Get-Date) - $t0).TotalSeconds -lt 10) { Start-Sleep -Milliseconds 300 }
  }
  [DS]::CurrentPercent()
}
