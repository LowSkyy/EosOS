# Verifica S5 delle impostazioni: i punti d'ingresso. La voce «Impostazioni di EosOS…» nel menu del nome nella barra
# (sul desktop, e quando il menu globale e spento o l'app e esclusa), nel menu col clic destro della barra e nel menu
# del Dock. Per i menu nativi si clicca con il mouse l'ultima riga SOLO del menu di EosOS appena aperto (verificato che sia suo):
# si clicca l'ultima riga. I quattro file delle impostazioni sono salvati e ripristinati.
# Finestre di prova NUOVE; nessuna finestra di altri toccata.
#
# Uso: powershell -File scripts\test-settings-s5.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class S5 {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint p);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern IntPtr GetShellWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  // Un menu a comparsa (classe #32768) visibile di uno dei processi indicati.
  public static long PopupOf(uint[] pids) { long f = 0; EnumWindows((h, l) => { if (!IsWindowVisible(h)) return true; var c = new StringBuilder(64); GetClassName(h, c, 64); if (c.ToString() != "#32768") return true; uint p; GetWindowThreadProcessId(h, out p); if (Array.IndexOf(pids, p) >= 0) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  // Clic al centro della riga piu in basso del menu (l'ultima voce).
  public static void ClickLastItem(long h) { R r; GetWindowRect((IntPtr)h, out r); SetCursorPos((r.L + r.Rt) / 2, r.B - 14); System.Threading.Thread.Sleep(250); mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Tap(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$expr) { Js 'launcher' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Row { Inv "window.__TAURI_INTERNALS__.invoke('menubar_current')" }
function Settings-Open { [S5]::Find('EosOS Impostazioni') -ne 0 }
function Close-Settings { [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_close')"); Wait-Until { -not (Settings-Open) } 4 | Out-Null; Start-Sleep -Milliseconds 300 }
function Panel-Open { [S5]::Find('EosOS Pannello') -ne 0 }
function Panel-Items { @(Js 'panel' "Array.from(document.querySelectorAll('.menu-item')).map(function(e){return e.querySelector('.menu-title').textContent})" | ForEach-Object { $_ }) }
function Close-Panel { [void](Inv "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][S5]::SetForegroundWindow($h)
    if (Wait-Until { [S5]::GetForegroundWindow() -eq $h } 1) { return }
    [S5]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [S5]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12 }
function Eos-Pids { [uint32[]]@(Get-Process eosos -ErrorAction SilentlyContinue | ForEach-Object { [uint32]$_.Id }) }
# Apre il menu della barra e sceglie la voce dell'indice dato (0 = nome dell'app).
function Open-Bar-Menu([int]$i) { Close-Panel; Js 'bar' "document.querySelectorAll('.app-menu')[$i].click()" | Out-Null; Wait-Until { (Panel-Open) -and ((Panel-Items).Count -gt 0) } 6 | Out-Null }
function Choose-Settings-In-Panel { Js 'panel' "(Array.from(document.querySelectorAll('.menu-item')).find(function(e){return e.querySelector('.menu-title').textContent.indexOf('Impostazioni di EosOS')===0})||{click:function(){}}).click()" | Out-Null }
# Apre un menu nativo (comando di una pagina) e sceglie l'ULTIMA voce con la tastiera, dopo aver verificato che il menu sia di EosOS.
function Native-Last([string]$view, [string]$cmd) {
  [void](Js $view "(function(){window.__TAURI_INTERNALS__.invoke('$cmd');return 1})()")
  $pop = 0
  Wait-Until { $script:pop = [S5]::PopupOf((Eos-Pids)); $script:pop -ne 0 } 5 | Out-Null
  if ($script:pop -eq 0) { return $false }
  # Il menu e di EosOS (verificato sopra): si clicca la sua ultima riga.
  [S5]::ClickLastItem($script:pop)
  $true
}

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  Start-Process 'conhost.exe' -ArgumentList 'powershell.exe -NoProfile -NoExit -Command "$Host.UI.RawUI.WindowTitle=''EosOS-S5-C''"' | Out-Null
  $c = 0; Wait-Until { $script:c = [S5]::Find('EosOS-S5-C'); $script:c -ne 0 } 15 | Out-Null
  Check 'finestra di prova creata' ($c -ne 0)
  Start-Eos

  Write-Host 'A. menu del nome quando i menu dell app non si mostrano (menu globale spento)'
  [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'menu.enabled',value:false})")
  Focus ([IntPtr]$c); Start-Sleep -Milliseconds 400
  Check 'la riga e solo il nome' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$c) -and (@($r.menus).Count -eq 1) } 6)
  Check 'e il nome si puo aprire (ha una voce)' ((Row).menus[0].items.Count -eq 1)
  Open-Bar-Menu 0
  Check 'la tendina ha la voce Impostazioni di EosOS' ((@(Panel-Items)).Count -eq 1 -and (@(Panel-Items))[0] -like 'Impostazioni di EosOS*') ((@(Panel-Items)) -join '|')
  Choose-Settings-In-Panel
  Check 'sceglierla apre la finestra delle impostazioni' (Wait-Until { Settings-Open } 5)
  Close-Settings
  [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'menu.enabled',value:true})")

  Write-Host 'B. sul desktop'
  Focus ([S5]::GetShellWindow()); Start-Sleep -Milliseconds 800
  Check 'la riga e quella del Desktop, con il solo nome' (Wait-Until { $r = Row; ($r.name -eq 'Desktop') -and (@($r.menus).Count -eq 1) } 6) ((Row).name)
  Open-Bar-Menu 0
  Check 'la tendina del nome ha la voce delle impostazioni' ((@(Panel-Items)).Count -eq 1 -and (@(Panel-Items))[0] -like 'Impostazioni di EosOS*')
  Choose-Settings-In-Panel
  Check 'e la apre' (Wait-Until { Settings-Open } 5)
  Close-Settings

  Write-Host 'C. menu nativo col clic destro sulla barra'
  Focus ([IntPtr]$c); Start-Sleep -Milliseconds 400
  $ok = Native-Last 'bar' 'indicators_menu'
  Check 'il menu nativo della barra si apre ed e di EosOS' $ok
  Check 'l'ultima voce apre le impostazioni' (Wait-Until { Settings-Open } 5)
  Close-Settings

  Write-Host 'D. menu nativo del Dock'
  $ok = Native-Last 'dock' 'dock_settings_menu'
  Check 'il menu nativo del Dock si apre ed e di EosOS' $ok
  Check 'l'ultima voce apre le impostazioni' (Wait-Until { Settings-Open } 5)
  Check 'il menu non e rimasto aperto' ([S5]::PopupOf((Eos-Pids)) -eq 0)
  Close-Settings
  Check 'dopo l apertura da menu le impostazioni sono ancora quelle (nessuna modifica per sbaglio)' ((@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq 'dock.autohide' }).value -eq $false)
}
finally {
  if ([S5]::PopupOf((Eos-Pids)) -ne 0) { [S5]::Tap(0x1B) }
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*EosOS-S5-C*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  foreach ($f in $files) {
    $p = Join-Path $dir $f
    Remove-Item $p -ErrorAction SilentlyContinue
    if ($null -ne $backup[$f]) { [System.IO.File]::WriteAllBytes($p, $backup[$f]) }
  }
}
$same = $true
foreach ($f in $files) { $p = Join-Path $dir $f; $now = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null }; if (($null -eq $now) -ne ($null -eq $backup[$f])) { $same = $false } elseif ($null -ne $now -and -not [System.Linq.Enumerable]::SequenceEqual([byte[]]$now, [byte[]]$backup[$f])) { $same = $false } }
Write-Host ("File dell utente ripristinati identici: " + $same)
if ($script:fail -eq 0 -and $same) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
