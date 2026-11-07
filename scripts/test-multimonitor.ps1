# Test multi-monitor dal vivo. NON nasconde la taskbar.
# Verifica: una barra per monitor (posizione, larghezza, altezza = 28 px * scala),
# area di lavoro ridotta su ogni monitor, e ripristino dopo la chiusura.
# Con -Hotplug simula lo scollegamento del secondo monitor DENTRO l'app (hook
# solo debug: file %TEMP%\eosos-hide-monitor). Non cambia la topologia di Windows:
# DisplaySwitch /internal non ha effetto in sessioni non interattive.
# Uso: powershell -File scripts\test-multimonitor.ps1 [-Hotplug]   (richiede `cargo build`)
param([switch]$Hotplug)
$ErrorActionPreference = 'Stop'
$exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path

Add-Type @'
using System; using System.Runtime.InteropServices; using System.Collections.Generic;
public static class M {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L,T,R,B; }
  [StructLayout(LayoutKind.Sequential, CharSet=CharSet.Unicode)] public struct MI { public int cb; public RECT mon; public RECT work; public uint flags; [MarshalAs(UnmanagedType.ByValTStr, SizeConst=32)] public string dev; }
  delegate bool MProc(IntPtr h, IntPtr dc, ref RECT r, IntPtr d);
  delegate bool WProc(IntPtr h, IntPtr p);
  [DllImport("user32.dll")] static extern bool EnumDisplayMonitors(IntPtr dc, IntPtr clip, MProc p, IntPtr d);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern bool GetMonitorInfo(IntPtr h, ref MI mi);
  [DllImport("shcore.dll")] static extern int GetDpiForMonitor(IntPtr h, int t, out uint x, out uint y);
  [DllImport("user32.dll")] static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  [DllImport("user32.dll")] static extern bool EnumWindows(WProc p, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int n);
  public static void Init() { SetProcessDpiAwarenessContext(new IntPtr(-4)); }
  public static void ShowTaskbars() { ShowWindow(FindWindow("Shell_TrayWnd", null), 8); }
  public static string Monitors() {
    var o = new List<string>();
    EnumDisplayMonitors(IntPtr.Zero, IntPtr.Zero, (IntPtr h, IntPtr dc, ref RECT r, IntPtr d) => {
      var mi = new MI(); mi.cb = Marshal.SizeOf(typeof(MI)); GetMonitorInfo(h, ref mi);
      uint x, y; GetDpiForMonitor(h, 0, out x, out y);
      o.Add(string.Join(",", mi.dev, mi.mon.L, mi.mon.T, mi.mon.R, mi.mon.B, mi.work.L, mi.work.T, mi.work.R, mi.work.B, x));
      return true; }, IntPtr.Zero);
    return string.Join(";", o);
  }
  public static string Bars(uint pid) {
    var o = new List<string>();
    EnumWindows((IntPtr h, IntPtr p) => {
      uint wp; GetWindowThreadProcessId(h, out wp);
      RECT r;
      if (wp == pid && IsWindowVisible(h) && GetWindowRect(h, out r) && (r.B - r.T) < 120 && (r.R - r.L) > 800)
        o.Add(string.Join(",", r.L, r.T, r.R, r.B));
      return true; }, IntPtr.Zero);
    return string.Join(";", o);
  }
}
'@
[M]::Init()

function Mons { ([M]::Monitors() -split ';' | Where-Object { $_ }) | ForEach-Object {
    $f = $_ -split ','; [pscustomobject]@{ Dev=$f[0]; L=[int]$f[1]; T=[int]$f[2]; R=[int]$f[3]; B=[int]$f[4]; WL=[int]$f[5]; WT=[int]$f[6]; WR=[int]$f[7]; WB=[int]$f[8]; Dpi=[int]$f[9] } } }
function Bars($procId) { ([M]::Bars($procId) -split ';' | Where-Object { $_ }) | ForEach-Object {
    $f = $_ -split ','; [pscustomobject]@{ L=[int]$f[0]; T=[int]$f[1]; R=[int]$f[2]; B=[int]$f[3] } } }
function MainPid { (Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notmatch '--watchdog' } | Select-Object -First 1).ProcessId }
function Cleanup { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }

$fail = 0
function Check($label, $cond) {
    if ($cond) { Write-Host "[OK]      $label" } else { Write-Host "[FALLITO] $label" -ForegroundColor Red; $script:fail++ }
}

# Controlla barre e area di lavoro per tutti i monitor presenti.
function CheckBars($tag) {
    $mons = @(Mons); $procId = MainPid; $bars = @(Bars $procId)
    Write-Host ("  [{0}] monitor={1} barre={2}" -f $tag, $mons.Count, $bars.Count)
    Check "$tag`: una barra per monitor" ($bars.Count -eq $mons.Count)
    foreach ($m in $mons) {
        $h = [int][math]::Round(28 * $m.Dpi / 96)
        $b = $bars | Where-Object { $_.L -eq $m.L -and $_.T -eq $m.T } | Select-Object -First 1
        Check "$tag`: $($m.Dev) barra in cima ($($m.L),$($m.T)) larga $($m.R - $m.L)" ($null -ne $b -and ($b.R - $b.L) -eq ($m.R - $m.L))
        Check "$tag`: $($m.Dev) altezza barra = $h px (dpi $($m.Dpi))" ($null -ne $b -and ($b.B - $b.T) -eq $h)
        Check "$tag`: $($m.Dev) area di lavoro inizia sotto la barra" ($m.WT -eq ($m.T + $h))
    }
}
function CheckRestored($tag) {
    foreach ($m in (Mons)) { Check "$tag`: $($m.Dev) area di lavoro senza barra" ($m.WT -eq $m.T) }
}

try {
    Cleanup; Start-Sleep 1
    Write-Host "Monitor presenti:"; Mons | Format-Table -AutoSize | Out-String | Write-Host
    $startCount = @(Mons).Count

    Write-Host "=== 1. Avvio e verifica delle barre ==="
    Start-Process $exe | Out-Null
    Start-Sleep 8
    CheckBars 'avvio'

    if ($Hotplug -and $startCount -ge 2) {
        # Scollegamento simulato DENTRO l'app (build debug): l'app ignora il monitor
        # indicato in %TEMP%\eosos-hide-monitor. Non cambia la topologia di Windows.
        $second = (Mons | Where-Object { $_.L -ne 0 -or $_.T -ne 0 } | Select-Object -First 1).Dev
        $flag = Join-Path $env:TEMP 'eosos-hide-monitor'

        Write-Host "`n=== 2. Monitor $second 'scollegato' (simulato nell'app) ==="
        Set-Content $flag $second
        Start-Sleep 4
        $procId = MainPid; $bars = @(Bars $procId)
        Check "scollegato: resta 1 sola barra" ($bars.Count -eq 1)
        $m2 = Mons | Where-Object { $_.Dev -eq $second }
        Check "scollegato: area di lavoro di $second senza barra" ($m2.WT -eq $m2.T)
        $m1 = Mons | Where-Object { $_.Dev -ne $second } | Select-Object -First 1
        Check "scollegato: l'altro monitor mantiene la barra" ($m1.WT -gt $m1.T)

        Write-Host "`n=== 3. Monitor ricollegato ==="
        Remove-Item $flag -ErrorAction SilentlyContinue
        Start-Sleep 4
        CheckBars 'ricollegato'
    }

    Write-Host "`n=== Chiusura: area di lavoro ripristinata su tutti i monitor ==="
    Stop-Process -Id (MainPid) -Force
    Start-Sleep 4
    CheckRestored 'dopo-kill'
}
finally {
    Cleanup
    Remove-Item (Join-Path $env:TEMP 'eosos-hide-monitor') -ErrorAction SilentlyContinue
    [M]::ShowTaskbars()
}

if ($fail -gt 0) { Write-Host "`n$fail controlli FALLITI" -ForegroundColor Red; exit 1 }
Write-Host "`nTUTTI I CONTROLLI OK" -ForegroundColor Green
