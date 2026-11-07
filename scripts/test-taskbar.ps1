# Test end-to-end del ripristino taskbar sulla VERA taskbar.
# Nasconde la taskbar per pochi secondi per scenario; alla fine la ripristina sempre.
# Uso: powershell -File scripts\test-taskbar.ps1   (richiede `cargo build` in src-tauri)
param([string]$Exe, [switch]$SkipPanic)   # -Exe: percorso di un eosos.exe (es. release); -SkipPanic: l'hook di panic esiste solo in debug
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = Resolve-Path $Exe } else { $exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path }

Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class W {
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
}
'@

function TaskbarVisible { [W]::IsWindowVisible([W]::FindWindow('Shell_TrayWnd', $null)) }
function ForceRestore { [void][W]::ShowWindow([W]::FindWindow('Shell_TrayWnd', $null), 8) }
function EosProcs { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" }
function MainProc { EosProcs | Where-Object { $_.CommandLine -notmatch '--watchdog' } | Select-Object -First 1 }
function Cleanup { EosProcs | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }

$results = @()
function Scenario($name, $envVars, [scriptblock]$trigger) {
    Write-Host "`n=== $name ==="
    Cleanup; Start-Sleep 1
    foreach ($k in $envVars.Keys) { Set-Item "env:$k" $envVars[$k] }
    $env:EOSOS_HIDE_TASKBAR = '1'
    $p = Start-Process $exe -PassThru
    Start-Sleep 6
    $hiddenOk = -not (TaskbarVisible)
    Write-Host ("taskbar nascosta all'avvio: {0}" -f $hiddenOk)
    & $trigger $p
    Start-Sleep 3
    $restored = TaskbarVisible
    Write-Host ("taskbar visibile dopo lo scenario: {0}" -f $restored)
    $script:results += [pscustomobject]@{ Scenario = $name; NascostaAvvio = $hiddenOk; Ripristinata = $restored }
    foreach ($k in $envVars.Keys) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
    Remove-Item env:EOSOS_HIDE_TASKBAR -ErrorAction SilentlyContinue
    Cleanup; Start-Sleep 1; ForceRestore
}

try {
    Scenario '1. Uscita normale (chiusura finestra)' @{} {
        param($p) $m = MainProc; (Get-Process -Id $m.ProcessId).CloseMainWindow() | Out-Null
    }
    Scenario '2. Kill brutale del processo principale' @{} {
        param($p) $m = MainProc; Stop-Process -Id $m.ProcessId -Force
    }
    if (-not $SkipPanic) {
    Scenario '3. Panic (hook)' @{ EOSOS_TEST_PANIC = '1' } {
        param($p) Start-Sleep 4   # il panic scatta da solo a 9 s (controllo nascosto a 6 s)
    }
    }
    Scenario '4. Scorciatoia Ctrl+Alt+Shift+Esc' @{} {
        param($p)
        foreach ($vk in 0x11, 0x12, 0x10, 0x1B) { [W]::keybd_event($vk, 0, 0, [UIntPtr]::Zero) }
        foreach ($vk in 0x1B, 0x10, 0x12, 0x11) { [W]::keybd_event($vk, 0, 2, [UIntPtr]::Zero) }
    }
}
finally {
    Cleanup
    ForceRestore
}

Write-Host "`n--- RIEPILOGO ---"
$results | Format-Table -AutoSize
$bad = @($results | Where-Object { (-not $_.Ripristinata) -or (-not $_.NascostaAvvio) })
if ($bad.Count -gt 0 -or $results.Count -ne $(if ($SkipPanic) { 3 } else { 4 })) {
    Write-Host 'FALLITO' -ForegroundColor Red; exit 1
}
Write-Host 'TUTTI GLI SCENARI OK' -ForegroundColor Green
