# Riavvia explorer.exe mentre EosOS e' attivo con la taskbar nascosta e verifica che
# la nuova taskbar venga ri-nascosta e che la barra EosOS si ri-registri (area di lavoro).
# ATTENZIONE: riavvia davvero Explorer (desktop e taskbar spariscono per qualche secondo e le
# finestre di Esplora risorse si chiudono).
#
# Uso: powershell -File scripts\test-explorer-restart.ps1 [-Exe percorso\eosos.exe] [-Dock]
#   -Dock: controlla anche il Dock (deve restare visibile, nello stesso punto, e tornare a
#          riservare lo spazio in basso). Senza -Dock la riserva del Dock e' spenta e si misura
#          l'area di lavoro della sola barra in alto.
param([string]$Exe, [switch]$Dock)
$ErrorActionPreference = 'Stop'
if (-not $Dock) {
  # Il Dock riserva una fascia in basso (appbar): qui si misura l'area di lavoro della sola barra in alto.
  $env:EOSOS_DOCK_RESERVE = '0'
} else { Remove-Item env:EOSOS_DOCK_RESERVE -ErrorAction SilentlyContinue }
if ($Exe) { $exe = Resolve-Path $Exe } else { $exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path }
$marker = Join-Path $env:LOCALAPPDATA 'EosOS\autohide-pending'
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class E {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref RECT r, uint c);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  public static RECT Work() { RECT r = new RECT(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, System.Text.StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  public static IntPtr FindTitle(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new System.Text.StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  public static string DockState() { IntPtr h = FindTitle("EosOS Dock"); if (h == IntPtr.Zero) return "assente"; RECT r; GetWindowRect(h, out r); return (IsWindowVisible(h) ? "visibile" : "NASCOSTO") + " " + r.L + "," + r.T + "," + r.R + "," + r.B; }
}
'@
function Cleanup { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function TbVisible { [E]::IsWindowVisible([E]::FindWindow('Shell_TrayWnd', $null)) }
$fail = 0
function Check($l, $c) { if ($c) { Write-Host "[OK]    $l" } else { Write-Host "[FALLITO] $l" -ForegroundColor Red; $script:fail++ } }
try {
  Cleanup; Start-Sleep 1
  $env:EOSOS_HIDE_TASKBAR='1'; $p = Start-Process $exe -PassThru; Remove-Item env:EOSOS_HIDE_TASKBAR
  Start-Sleep 9
  $before = [E]::Work(); Write-Host "prima: area di lavoro $($before.T)..$($before.B), taskbar visibile: $(TbVisible)"
  Check "prima: taskbar nascosta" (-not (TbVisible))
  Check "prima: barra registrata (top > 0)" ($before.T -gt 0)
  if ($Dock) {
    $dockBefore = [E]::DockState(); Write-Host "prima: Dock $dockBefore"
    Check "prima: Dock visibile" ($dockBefore -like 'visibile*')
  }

  Write-Host "`n--- riavvio di explorer.exe ---"
  Stop-Process -Name explorer -Force
  Start-Sleep 3
  if (-not (Get-Process explorer -ErrorAction SilentlyContinue)) { Start-Process explorer }
  Start-Sleep 12
  $after = [E]::Work(); Write-Host "dopo: area di lavoro $($after.T)..$($after.B), taskbar visibile: $(TbVisible)"
  Check "dopo: taskbar ancora nascosta" (-not (TbVisible))
  Check "dopo: barra ri-registrata (top = prima)" ($after.T -eq $before.T)
  Check "dopo: nessuna fascia vuota in basso (B = prima)" ($after.B -eq $before.B)
  if ($Dock) {
    $dockAfter = [E]::DockState(); Write-Host "dopo: Dock $dockAfter"
    Check "dopo: Dock ancora visibile" ($dockAfter -like 'visibile*')
    # La larghezza può cambiare: il riavvio chiude le finestre di Esplora risorse e la sua icona sparisce.
    # Contano il bordo inferiore e il centro.
    function Parse-Dock($d) { if ($d -match '(\d+),(-?\d+),(\d+),(-?\d+)$') { [pscustomobject]@{ L = [int]$Matches[1]; T = [int]$Matches[2]; R = [int]$Matches[3]; B = [int]$Matches[4] } } }
    $db = Parse-Dock $dockBefore; $da = Parse-Dock $dockAfter
    Check "dopo: Dock allo stesso livello (bordo inferiore)" ($db -and $da -and $db.B -eq $da.B)
    Check "dopo: Dock ancora centrato (centro orizzontale)" ($db -and $da -and [Math]::Abs(($db.L + $db.R) - ($da.L + $da.R)) -le 2)
  }

  Write-Host "`n--- uscita normale ---"
  $m = Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notmatch '--watchdog' } | Select-Object -First 1
  [void](Get-Process -Id $m.ProcessId).CloseMainWindow(); Start-Sleep 4
  Check "uscita: taskbar visibile" (TbVisible)
  Check "uscita: marcatore rimosso" (-not (Test-Path $marker))
} finally {
  Cleanup
  Start-Sleep 1
  if (-not (Get-Process explorer -ErrorAction SilentlyContinue)) { Start-Process explorer; Start-Sleep 5 }
  [void][E]::ShowWindow([E]::FindWindow('Shell_TrayWnd', $null), 8)
}
if ($fail -gt 0) { Write-Host "`n$fail controlli falliti" -ForegroundColor Red; exit 1 } else { Write-Host "`nTutto OK" }
