# Verifica le barre alla scala (DPI) CORRENTE di ogni monitor. NON cambia la scala di Windows
# e NON nasconde la taskbar: la scala la imposti tu in Impostazioni > Schermo > Scala.
# Esegue test-multimonitor.ps1 (altezza barra = round(28 * dpi/96), larghezza, area di lavoro)
# e aggiunge una riga al report %TEMP%\eosos-dpi-report.txt, cosi' puoi provare piu' scale
# (100%, 125%, 150%, 175%...) e confrontare i risultati. Richiede `cargo build` in src-tauri.
# Uso: powershell -File scripts\test-dpi.ps1
$ErrorActionPreference = 'Stop'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class D {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct MI { public int cb; public RECT mon; public RECT work; public uint flags; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dev; }
  delegate bool MProc(IntPtr h, IntPtr dc, ref RECT r, IntPtr d);
  [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, MProc p, IntPtr d);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetMonitorInfo(IntPtr h, ref MI mi);
  [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr h, int t, out uint x, out uint y);
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  public static void Init() { SetProcessDpiAwarenessContext(new IntPtr(-4)); }
  public static string Scales() {
    var o = new List<string>();
    EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (IntPtr h, IntPtr dc, ref RECT r, IntPtr d) => {
      var mi = new MI(); mi.cb = Marshal.SizeOf(typeof(MI)); GetMonitorInfo(h, ref mi);
      uint x, y; GetDpiForMonitor(h, 0, out x, out y);
      o.Add(mi.dev + "=" + (x * 100 / 96) + "%");
      return true; }, IntPtr.Zero);
    return string.Join(" ", o);
  }
}
'@
[D]::Init()
$scales = [D]::Scales()
Write-Host "Scale correnti: $scales"
$non100 = ($scales -split ' ') | Where-Object { $_ -notmatch '=100%$' }
if (-not $non100) {
    Write-Host "ATTENZIONE: tutti i monitor sono al 100%. Il controllo e' valido ma non prova nulla di nuovo:" -ForegroundColor Yellow
    Write-Host "imposta una scala diversa (es. 125% o 150%) in Impostazioni > Sistema > Schermo e rilancia." -ForegroundColor Yellow
}
& powershell -NoProfile -ExecutionPolicy Bypass -File (Join-Path $PSScriptRoot 'test-multimonitor.ps1')
$code = $LASTEXITCODE
$esito = if ($code -eq 0) { 'OK' } else { 'FALLITO' }
$line = "{0}  scale: {1}  esito: {2}" -f (Get-Date -Format 'yyyy-MM-dd HH:mm'), $scales, $esito
Add-Content -Path (Join-Path $env:TEMP 'eosos-dpi-report.txt') -Value $line -Encoding utf8
Write-Host "`nReport ($env:TEMP\eosos-dpi-report.txt):"
Get-Content (Join-Path $env:TEMP 'eosos-dpi-report.txt') | Write-Host
exit $code
