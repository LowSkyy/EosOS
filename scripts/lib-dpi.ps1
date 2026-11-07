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

  [StructLayout(LayoutKind.Sequential)] public struct RC { public int L, T, R, B; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct MIX { public int cb; public RC mon; public RC work; public uint flags; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dev; }
  delegate bool MP(IntPtr h, IntPtr dc, ref RC r, IntPtr d);
  [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, MP p, IntPtr d);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetMonitorInfo(IntPtr h, ref MIX mi);

  // Un monitor per riga: "nome GDI|scala%|sinistra,alto,destra,basso|primario(0/1)|area di lavoro" (pixel fisici).
  public static string[] Monitors() {
    var o = new System.Collections.Generic.List<string>();
    EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (IntPtr h, IntPtr dc, ref RC r, IntPtr d) => {
      var mi = new MIX(); mi.cb = Marshal.SizeOf(typeof(MIX)); GetMonitorInfo(h, ref mi);
      uint x, y; GetDpiForMonitor(h, 0, out x, out y);
      o.Add(mi.dev + "|" + (x * 100 / 96) + "|" + mi.mon.L + "," + mi.mon.T + "," + mi.mon.R + "," + mi.mon.B + "|" + ((mi.flags & 1) != 0 ? 1 : 0) + "|" + mi.work.L + "," + mi.work.T + "," + mi.work.R + "," + mi.work.B);
      return true; }, IntPtr.Zero);
    return o.ToArray();
  }

  // Scala (percentuale) del monitor con questo nome GDI, o 0 se non c'e'.
  public static int PercentOf(string gdi) {
    foreach (var line in Monitors()) { var f = line.Split('|'); if (f[0] == gdi) return int.Parse(f[1]); }
    return 0;
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

# Tutti i monitor attivi: Gdi, Scala, rettangolo (L, T, R, B), Primario e area di lavoro (WorkL..WorkB), in pixel fisici.
function Get-Monitors {
  [DS]::Monitors() | ForEach-Object {
    $f = $_ -split '\|'; $r = $f[2] -split ','; $w = $f[4] -split ','
    [pscustomobject]@{ Gdi = $f[0]; Scala = [int]$f[1]; L = [int]$r[0]; T = [int]$r[1]; R = [int]$r[2]; B = [int]$r[3]; Primario = ($f[3] -eq '1'); WorkL = [int]$w[0]; WorkT = [int]$w[1]; WorkR = [int]$w[2]; WorkB = [int]$w[3] }
  }
}

function Get-MonitorScale([string]$Gdi) { [DS]::PercentOf($Gdi) }

# Scale supportate da un monitor (percentuali), o $null se la configurazione non e' leggibile.
function Get-MonitorScaleRange([string]$Gdi) {
  $rel = [DS]::Rel($Gdi); if (-not $rel) { return $null }
  $cur = [DS]::PercentOf($Gdi); $curIdx = [Array]::IndexOf([DS]::Steps, $cur)
  if ($curIdx -lt 0) { return $null }
  $rec = $curIdx - $rel[1]                       # indice della scala raccomandata
  ($rel[0]..$rel[2]) | ForEach-Object { $i = $rec + $_; if ($i -ge 0 -and $i -lt [DS]::Steps.Length) { [DS]::Steps[$i] } }
}

# Imposta la scala di un monitor. Restituisce la scala effettiva dopo il cambio.
function Set-MonitorScale([string]$Gdi, [int]$Percent) {
  $rel = [DS]::Rel($Gdi); if (-not $rel) { throw "configurazione dello schermo non leggibile ($Gdi)" }
  $cur = [DS]::PercentOf($Gdi); $curIdx = [Array]::IndexOf([DS]::Steps, $cur)
  $tgtIdx = [Array]::IndexOf([DS]::Steps, $Percent)
  if ($curIdx -lt 0 -or $tgtIdx -lt 0) { throw "scala non gestita ($cur% -> $Percent%)" }
  $rec = $curIdx - $rel[1]
  $want = $tgtIdx - $rec
  if ($want -lt $rel[0] -or $want -gt $rel[2]) { throw "il monitor $Gdi non supporta il $Percent%" }
  if ($want -ne $rel[1]) {
    if (-not [DS]::SetRel($Gdi, $want)) { throw "impostazione del $Percent% rifiutata" }
    $t0 = Get-Date
    while ([DS]::PercentOf($Gdi) -ne $Percent -and ((Get-Date) - $t0).TotalSeconds -lt 10) { Start-Sleep -Milliseconds 300 }
  }
  [DS]::PercentOf($Gdi)
}

function Get-PrimaryScale { [DS]::CurrentPercent() }
function Get-PrimaryScaleRange { Get-MonitorScaleRange (Get-PrimaryGdi) }
function Set-PrimaryScale([int]$Percent) { Set-MonitorScale (Get-PrimaryGdi) $Percent }
