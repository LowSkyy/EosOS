# Verifica che con la taskbar nascosta non resti la fascia vuota (auto-nascondi temporaneo)
# e che l'impostazione dell'utente venga ripristinata in ogni scenario.
# ATTENZIONE: nasconde la vera taskbar e cambia temporaneamente l'auto-nascondi. Richiede `cargo build`.
$ErrorActionPreference = 'Stop'
$exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path
$marker = Join-Path $env:LOCALAPPDATA 'EosOS\autohide-pending'
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref RECT r, uint c);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  public static RECT Work() { RECT r = new RECT(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
}
'@
function Cleanup { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Procs { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" }
function MainProc { Procs | Where-Object { $_.CommandLine -notmatch '--watchdog' } | Select-Object -First 1 }
function TbVisible { [W]::IsWindowVisible([W]::FindWindow('Shell_TrayWnd', $null)) }
$fail = 0
function Check($l, $c) { if ($c) { Write-Host "[OK]    $l" } else { Write-Host "[FALLITO] $l" -ForegroundColor Red; $script:fail++ } }
function Start-Hidden { $env:EOSOS_HIDE_TASKBAR='1'; $p = Start-Process $exe -PassThru; Remove-Item env:EOSOS_HIDE_TASKBAR; Start-Sleep 8; $p }

try {
  Cleanup; Start-Sleep 1
  Remove-Item $marker -ErrorAction SilentlyContinue
  $base = [W]::Work(); Write-Host "area di lavoro iniziale: $($base.T)..$($base.B)"

  Write-Host "`n=== A. taskbar nascosta: niente fascia vuota ==="
  [void](Start-Hidden)
  $w = [W]::Work(); Write-Host "area di lavoro: $($w.T)..$($w.B)"
  Check "taskbar nascosta" (-not (TbVisible))
  Check "l'area di lavoro arriva in fondo (B > iniziale)" ($w.B -gt $base.B)
  Check "marcatore auto-nascondi presente" (Test-Path $marker)
  [void](New-Object -ComObject WScript.Shell)
  $m = Get-Process -Id (MainProc).ProcessId
  [void]$m.CloseMainWindow(); Start-Sleep 4
  $a = [W]::Work()
  Check "uscita normale: taskbar visibile" (TbVisible)
  Check "uscita normale: area di lavoro come prima" ($a.T -eq $base.T -and $a.B -eq $base.B)
  Check "uscita normale: marcatore rimosso" (-not (Test-Path $marker))
  Cleanup; Start-Sleep 1

  Write-Host "`n=== B. kill del processo principale ==="
  [void](Start-Hidden)
  Stop-Process -Id (MainProc).ProcessId -Force; Start-Sleep 5
  $a = [W]::Work(); Write-Host "area di lavoro: $($a.T)..$($a.B)"
  Check "kill: taskbar visibile" (TbVisible)
  Check "kill: area di lavoro come prima" ($a.T -eq $base.T -and $a.B -eq $base.B)
  Check "kill: marcatore rimosso" (-not (Test-Path $marker))
  Cleanup; Start-Sleep 1

  Write-Host "`n=== C. kill simultaneo di EosOS e watchdog, poi riavvio (recupero) ==="
  [void](Start-Hidden)
  Cleanup; Start-Sleep 3
  Write-Host "taskbar visibile dopo kill totale: $(TbVisible) (atteso: no, limite noto)"
  Check "marcatore presente dopo kill totale" (Test-Path $marker)
  $p = Start-Process $exe -PassThru; Start-Sleep 6   # avvio senza nascondere: deve recuperare
  Cleanup; Start-Sleep 1
  [void][W]::ShowWindow([W]::FindWindow('Shell_TrayWnd', $null), 8)
  $a = [W]::Work()
  Check "recupero all'avvio: marcatore rimosso" (-not (Test-Path $marker))
  Check "recupero all'avvio: area di lavoro come prima" ($a.T -eq $base.T -and $a.B -eq $base.B)
}
finally {
  Cleanup
  [void][W]::ShowWindow([W]::FindWindow('Shell_TrayWnd', $null), 8)
}
if ($fail -gt 0) { Write-Host "`n$fail controlli falliti" -ForegroundColor Red; exit 1 } else { Write-Host "`nTutto OK" }
