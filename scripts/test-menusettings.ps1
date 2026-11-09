# Verifica G6 del menu globale: impostazioni (acceso/spento, app escluse) dalle azioni del Launcher, salvate in
# mission.json e valide dopo il riavvio; nessuna lettura per le app escluse. Due finestre di prova NUOVE (A con
# menu classico, B con MenuStrip). Il file mission.json dell'utente viene salvato e ripristinato.
# Nessun clic sulle finestre di altri, nessun tasto.
#
# Uso: powershell -File scripts\test-menusettings.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MS {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Row { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('menubar_current')" }
function Act([string]$id) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'$id'})" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][MS]::SetForegroundWindow($h)
    if (Wait-Until { [MS]::GetForegroundWindow() -eq $h } 1) { Start-Sleep -Milliseconds 700; return }   # la barra segue il primo piano entro 300 ms: si lascia il tempo
    [MS]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [MS]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12 }
function Count-Menus { @((Row).menus).Count }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = $null; if (Test-Path $mc) { $mcBackup = Get-Content $mc -Raw }; Remove-Item $mc -ErrorAction SilentlyContinue
$formA = Join-Path $env:TEMP 'eosos-menuset-a.ps1'; $formB = Join-Path $env:TEMP 'eosos-menuset-b.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Imp-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
foreach ($n in 'File', 'Modifica') { $i = New-Object System.Windows.Forms.MenuItem $n; [void]$i.MenuItems.Add('Voce'); [void]$m.MenuItems.Add($i) }
$f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8
Set-Content $formB @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Imp-B'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MenuStrip
foreach ($n in 'File', 'Modifica') { $i = New-Object System.Windows.Forms.ToolStripMenuItem $n; [void]$i.DropDownItems.Add('Voce'); [void]$m.Items.Add($i) }
$f.MainMenuStrip = $m; $f.Controls.Add($m); [void]$f.ShowDialog()
'@ -Encoding UTF8

try {
  # Di base il menu Finestra e' nascosto: questa prova lo riguarda, quindi lo si accende nel file (salvato prima e ripristinato dopo).
  $mcSeed = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
  New-Item -ItemType Directory -Force (Split-Path $mcSeed) | Out-Null
  [System.IO.File]::WriteAllText($mcSeed, '{"menuShowWindow": true, "menuShowName": true}', (New-Object System.Text.UTF8Encoding($false)))
  Cleanup-Eos; Start-Sleep 1
  Write-Host 'Preparazione: due finestre di prova NUOVE'
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formA | Out-Null
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formB | Out-Null
  $ids = @{}
  Wait-Until { $ids.A = [MS]::Find('EosOS-Imp-A'); $ids.B = [MS]::Find('EosOS-Imp-B'); ($ids.A -ne 0) -and ($ids.B -ne 0) } 20 | Out-Null
  Check 'finestre di prova create' (($ids.A -ne 0) -and ($ids.B -ne 0))
  Start-Eos

  Write-Host 'A. di default il menu globale e acceso'
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 600; Focus ([IntPtr]$ids.A)
  Check 'A mostra nome, File, Modifica, Finestra' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) -and ((Count-Menus) -eq 4) } 10) ("menu " + (Count-Menus))

  Write-Host 'B. spegnerlo dal Launcher'
  Check 'azione accettata' ((Act 'action:menu-toggle') -eq 'done')
  Check 'ora la barra mostra solo il nome dell app' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ((Count-Menus) -eq 1) } 10) ("menu " + (Count-Menus))
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 1000
  Check 'anche per le altre app' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ((Count-Menus) -eq 1) } 10)
  Check 'la scelta e nel file' ((Get-Content $mc -Raw) -match '"menuOff":\s*true')

  Write-Host 'C. riaccenderlo'
  Check 'azione accettata' ((Act 'action:menu-toggle') -eq 'done')
  Check 'B torna con i suoi menu (File, Modifica, Finestra + nome)' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ($r.read -eq $true) -and ((Count-Menus) -eq 4) } 10) ("menu " + (Count-Menus))

  Write-Host 'D. escludere solo l app in primo piano (B)'
  Check 'azione accettata' ((Act 'action:menu-exclude') -eq 'done')
  Check 'B mostra solo il nome' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ((Count-Menus) -eq 1) } 10) ("menu " + (Count-Menus))
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 1000
  Check 'anche A e esclusa: l esclusione e per PROGRAMMA (A e B sono lo stesso programma, PowerShell)' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ((Count-Menus) -eq 1) } 10) ("menu " + (Count-Menus))
  $f = Get-Content $mc -Raw
  Check 'l esclusione e nel file (nome dell app, non titoli di finestra)' ($f -match '"menuExcluded"' -and $f -match 'PowerShell')

  Write-Host 'E. dopo il riavvio le scelte restano'
  Cleanup-Eos; Start-Sleep 2; Start-Eos
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 600; Focus ([IntPtr]$ids.A)
  Check 'dopo il riavvio: l esclusione vale per PowerShell (stesso programma di A e B)' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ((Count-Menus) -eq 1) } 10) ("menu " + (Count-Menus))

  Write-Host 'F. reincluderla'
  Check 'azione accettata' ((Act 'action:menu-exclude') -eq 'done')
  Check 'A torna con i suoi menu' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) -and ((Count-Menus) -eq 4) } 10) ("menu " + (Count-Menus))
}
finally {
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menuset-*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Remove-Item $formA, $formB -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($mcBackup) { Set-Content $mc $mcBackup -NoNewline -Encoding UTF8 }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
