# Test dell'appbar: misura l'area di lavoro di Windows (SPI_GETWORKAREA).
# Non nasconde la taskbar (nessuna EOSOS_HIDE_TASKBAR) tranne nello scenario "kill con taskbar".
# Uso: powershell -File scripts\test-appbar.ps1   (richiede `cargo build` in src-tauri)
$ErrorActionPreference = 'Stop'
$exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class A {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref RECT r, uint c);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int i);
  public static RECT Work() { RECT r = new RECT(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
}
'@

function Work { $r = [A]::Work(); "$($r.L),$($r.T),$($r.R),$($r.B)" }
function Cleanup { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function MainProc { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notmatch '--watchdog' } | Select-Object -First 1 }
function ForceRestoreTaskbar { [void][A]::ShowWindow([A]::FindWindow('Shell_TrayWnd', $null), 8) }

$fail = 0
function Check($label, $cond) {
    if ($cond) { Write-Host "[OK]    $label" } else { Write-Host "[FALLITO] $label" -ForegroundColor Red; $script:fail++ }
}

try {
    Cleanup; Start-Sleep 1
    $base = [A]::Work()
    Write-Host "area di lavoro iniziale: $(Work)"

    Write-Host "`n=== A. avvio, finestra e area di lavoro ==="
    $p = Start-Process $exe -PassThru
    Start-Sleep 6
    $w = [A]::Work()
    Write-Host "area di lavoro con la barra: $(Work)"
    Check "l'area di lavoro si restringe dall'alto (top > iniziale)" ($w.T -gt $base.T)
    $m = Get-Process -Id (MainProc).ProcessId
    $r = New-Object A+RECT
    [void][A]::GetWindowRect($m.MainWindowHandle, [ref]$r)
    Write-Host "rettangolo barra: $($r.L),$($r.T),$($r.R),$($r.B)"
    Check "la barra e' in cima (top = top monitor)" ($r.T -eq $base.T)
    Check "la barra occupa tutta la larghezza" (($r.R - $r.L) -ge ($base.R - $base.L))
    Check "area di lavoro inizia sotto la barra" ($w.T -eq $r.B)

    Write-Host "`n=== B. uscita normale: area di lavoro ripristinata ==="
    [void]$m.CloseMainWindow(); Start-Sleep 3
    $a = [A]::Work()
    Write-Host "area di lavoro dopo l'uscita: $(Work)"
    Check "area di lavoro ripristinata" ($a.T -eq $base.T -and $a.B -eq $base.B)
    Cleanup; Start-Sleep 1

    Write-Host "`n=== C. kill brutale (senza watchdog): l'area di lavoro si ripristina da sola? ==="
    $p = Start-Process $exe -PassThru
    Start-Sleep 6
    Stop-Process -Id (MainProc).ProcessId -Force
    Start-Sleep 4
    $a = [A]::Work()
    Write-Host "area di lavoro dopo il kill: $(Work)"
    Check "area di lavoro ripristinata dopo kill" ($a.T -eq $base.T -and $a.B -eq $base.B)
    Cleanup; Start-Sleep 1

    Write-Host "`n=== D. kill con taskbar nascosta (watchdog attivo) ==="
    $env:EOSOS_HIDE_TASKBAR = '1'
    $p = Start-Process $exe -PassThru
    Remove-Item env:EOSOS_HIDE_TASKBAR
    Start-Sleep 7
    Write-Host "area di lavoro con barra e taskbar nascosta: $(Work)"
    $tb = [A]::IsWindowVisible([A]::FindWindow('Shell_TrayWnd', $null))
    Check "taskbar nascosta" (-not $tb)
    Stop-Process -Id (MainProc).ProcessId -Force
    Start-Sleep 4
    $a = [A]::Work()
    Write-Host "area di lavoro dopo il kill: $(Work)"
    Check "taskbar di nuovo visibile" ([A]::IsWindowVisible([A]::FindWindow('Shell_TrayWnd', $null)))
    Check "area di lavoro ripristinata" ($a.T -eq $base.T -and $a.B -eq $base.B)
}
finally {
    Cleanup
    ForceRestoreTaskbar
}

if ($fail -gt 0) { Write-Host "`n$fail controlli FALLITI" -ForegroundColor Red; exit 1 }
Write-Host "`nTUTTI I CONTROLLI OK" -ForegroundColor Green
