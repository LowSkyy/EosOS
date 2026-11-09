# Verifica G4 del menu globale: la tendina dei menu della barra. Finestre di prova NUOVE: A con menu classico
# (HMENU), B con barra MenuStrip (menu pigri, UI Automation), C console classica. I clic sono sempre sulla barra di
# EosOS o sulla tendina di EosOS (mai su finestre dell'utente), via CDP o, una volta, col mouse vero DOPO aver
# verificato che sotto il cursore ci sia proprio il titolo del menu. I comandi di EosOS (riduci, ingrandisci,
# chiudi, esci) agiscono SOLO sulle mie finestre di prova. Nessun tasto verso finestre che non siano la tendina.
#
# Uso: powershell -File scripts\test-menudrop.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MD {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(POINT p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int x, y; }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new POINT { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$args2 = '') { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('$cmd'$args2)" }
function Row { Inv 'menubar_current' }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][MD]::SetForegroundWindow($h)
    if (Wait-Until { [MD]::GetForegroundWindow() -eq $h } 1) { return }
    [MD]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [MD]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Panel-Open { [MD]::Find('EosOS Pannello') -ne 0 }
function Panel-Items { @(Js 'panel' "Array.from(document.querySelectorAll('.menu-item')).map(function(e){return e.querySelector('.menu-title').textContent})" | ForEach-Object { $_ }) }
function Panel-Label { Js 'panel' "(document.querySelector('.menu-panel')||{getAttribute:function(){return null}}).getAttribute('aria-label')" }
function Bar-Click([int]$i) { Js 'bar' "document.querySelectorAll('.app-menu')[$i].click()" | Out-Null }
function Bar-Hover([int]$i) { Js 'bar' "document.querySelectorAll('.app-menu')[$i].dispatchEvent(new MouseEvent('mouseover',{bubbles:true}))" | Out-Null }
function Panel-Click([string]$title) { Js 'panel' "(Array.from(document.querySelectorAll('.menu-item')).find(function(e){return e.querySelector('.menu-title').textContent==='$title'})||{click:function(){}}).click()" | Out-Null }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null }
function Open-Menu([int]$i) { Close-Panel; Bar-Click $i; Wait-Until { (Panel-Open) -and ((Panel-Items).Count -gt 0) } 6 | Out-Null }

$formA = Join-Path $env:TEMP 'eosos-menudrop-a.ps1'; $formB = Join-Path $env:TEMP 'eosos-menudrop-b.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Tendina-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
foreach ($n in '&File', 'Modifica', 'Visualizza') { $i = New-Object System.Windows.Forms.MenuItem $n; [void]$i.MenuItems.Add('&Apri' + [char]9 + 'Ctrl+O'); [void]$i.MenuItems.Add('-'); $r = New-Object System.Windows.Forms.MenuItem 'Recenti'; [void]$r.MenuItems.Add('uno'); [void]$r.MenuItems.Add('due'); [void]$i.MenuItems.Add($r); $g = New-Object System.Windows.Forms.MenuItem 'Spenta'; $g.Enabled = $false; [void]$i.MenuItems.Add($g); $c = New-Object System.Windows.Forms.MenuItem 'Spuntata'; $c.Checked = $true; [void]$i.MenuItems.Add($c); [void]$m.MenuItems.Add($i) }
$f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8
Set-Content $formB @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Tendina-B'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MenuStrip
foreach ($n in 'File', 'Modifica', 'Aiuto') { $i = New-Object System.Windows.Forms.ToolStripMenuItem $n; [void]$i.DropDownItems.Add('Prima voce'); [void]$i.DropDownItems.Add('Seconda voce'); [void]$m.Items.Add($i) }
$f.MainMenuStrip = $m; $f.Controls.Add($m); [void]$f.ShowDialog()
'@ -Encoding UTF8
$mcPath = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = if (Test-Path $mcPath) { [System.IO.File]::ReadAllBytes($mcPath) } else { $null }

try {
  # Di base il menu Finestra e' nascosto: questa prova lo riguarda, quindi lo si accende nel file (salvato prima e ripristinato dopo).
  $mcSeed = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
  New-Item -ItemType Directory -Force (Split-Path $mcSeed) | Out-Null
  [System.IO.File]::WriteAllText($mcSeed, '{"menuShowWindow": true, "menuShowName": true}', (New-Object System.Text.UTF8Encoding($false)))
  Cleanup-Eos; Start-Sleep 1
  Write-Host 'Preparazione: finestre di prova NUOVE'
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formA | Out-Null
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formB | Out-Null
  Start-Process 'conhost.exe' -ArgumentList 'powershell.exe -NoProfile -NoExit -Command "$Host.UI.RawUI.WindowTitle=''EosOS-Tendina-C''"' | Out-Null
  $ids = @{}
  Wait-Until { $ids.A = [MD]::Find('EosOS-Tendina-A'); $ids.B = [MD]::Find('EosOS-Tendina-B'); $ids.C = [MD]::Find('EosOS-Tendina-C'); ($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0) } 20 | Out-Null
  Check 'finestre di prova create' (($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0))
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. tendina di un menu classico'
  Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 600; Focus ([IntPtr]$ids.A)
  Check 'la riga e di A, letta' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) } 6)
  Close-Panel; Bar-Click 1
  Open-Menu 1
  Check 'la tendina si apre (finestra del pannello visibile)' (Panel-Open)
  $items = Panel-Items
  Check 'File: Apri, Recenti, Spenta, Spuntata (il separatore non e una voce)' (($items -join '|') -eq 'Apri|Recenti|Spenta|Spuntata') ($items -join '|')
  Check 'etichetta del pannello = File' ((Panel-Label) -eq 'File')
  Check 'scorciatoia Ctrl+O a destra di Apri' ((Js 'panel' "document.querySelectorAll('.menu-item')[0].querySelector('.menu-short').textContent") -eq 'Ctrl+O')
  Check 'Spenta e disattivata' ((Js 'panel' "document.querySelectorAll('.menu-item')[2].disabled") -eq $true)
  Check 'Spuntata ha il segno di spunta' ((Js 'panel' "document.querySelectorAll('.menu-item')[3].querySelector('.menu-check').textContent.charCodeAt(0)") -eq 10003)
  Check 'c e un separatore' ((Js 'panel' "document.querySelectorAll('.menu-sep').length") -eq 1)

  Write-Host 'B. sottomenu sul posto'
  Panel-Click 'Recenti'; Start-Sleep -Milliseconds 400
  Check 'aprendo Recenti compaiono uno e due, rientrati' (((Panel-Items) -join '|') -eq 'Apri|Recenti|uno|due|Spenta|Spuntata') ((Panel-Items) -join '|')
  Panel-Click 'Recenti'; Start-Sleep -Milliseconds 300
  Check 'richiudendolo spariscono' (((Panel-Items) -join '|') -eq 'Apri|Recenti|Spenta|Spuntata')

  Write-Host 'C. passaggio del mouse da un titolo all altro, e secondo clic che chiude'
  Bar-Hover 2; Start-Sleep -Milliseconds 700
  Check 'passando su Modifica la tendina diventa quella di Modifica' ((Panel-Label) -eq 'Modifica') ("etichetta " + (Panel-Label))
  Bar-Click 2; Start-Sleep -Milliseconds 600
  Check 'un secondo clic sullo stesso titolo la chiude' (-not (Panel-Open))
  Start-Sleep -Milliseconds 600

  Write-Host 'D. clic col mouse vero sul titolo File (dopo aver verificato cosa c e sotto il cursore)'
  $sx = [int](Js 'bar' 'window.screenX'); $sy = [int](Js 'bar' 'window.screenY'); $dpr = [double](Js 'bar' 'window.devicePixelRatio')
  $b = Js 'bar' "(function(){var r=document.querySelectorAll('.app-menu')[1].getBoundingClientRect();return [r.left+r.width/2,r.top+r.height/2]})()"
  $px = [int]($sx + $b[0] * $dpr); $py = [int]($sy + $b[1] * $dpr)
  [void][MD]::SetCursorPos($px, $py); Start-Sleep -Milliseconds 400
  $under = Js 'bar' "(function(){var e=document.elementFromPoint($([int]$b[0]),$([int]$b[1]));return e&&e.closest('[data-menu-index]')?e.closest('[data-menu-index]').getAttribute('data-menu-index'):null})()"
  Check 'sotto il cursore c e proprio il titolo File della barra di EosOS' ("$under" -eq '1') ("indice $under")
  if ("$under" -eq '1') { [MD]::Click() }
  Check 'il clic vero apre la tendina' (Wait-Until { (Panel-Open) -and ((Panel-Items).Count -gt 0) } 5)
  Check 'e la mostra del menu File' ((Panel-Label) -eq 'File')

  Write-Host 'E. tastiera nella tendina (i tasti vanno alla tendina di EosOS in primo piano)'
  $pan = [IntPtr][MD]::Find('EosOS Pannello')
  Check 'la tendina ha il primo piano (i tasti non vanno ad altre app)' (Wait-Until { [MD]::GetForegroundWindow() -eq $pan } 3)
  if ([MD]::GetForegroundWindow() -eq $pan) {
    [MD]::keybd_event(0x28, 0, 0, [UIntPtr]::Zero); [MD]::keybd_event(0x28, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
    Check 'freccia giu: seleziona la prima voce (Apri)' ((Js 'panel' "(document.querySelector('.menu-item-sel')||{textContent:''}).querySelector('.menu-title').textContent") -eq 'Apri')
    [MD]::keybd_event(0x28, 0, 0, [UIntPtr]::Zero); [MD]::keybd_event(0x28, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
    Check 'un altro giu: Recenti (salta il separatore)' ((Js 'panel' "document.querySelector('.menu-item-sel .menu-title').textContent") -eq 'Recenti')
    [MD]::keybd_event(0x28, 0, 0, [UIntPtr]::Zero); [MD]::keybd_event(0x28, 0, 2, [UIntPtr]::Zero); Start-Sleep -Milliseconds 300
    Check 'e ancora giu: salta la voce disattivata, va su Spuntata' ((Js 'panel' "document.querySelector('.menu-item-sel .menu-title').textContent") -eq 'Spuntata')
    [MD]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [MD]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
    Check 'Esc chiude la tendina' (Wait-Until { -not (Panel-Open) } 3)
  }

  Write-Host 'F. scegliere una voce dell app: per ora non fa nulla (l invocazione e G5) e non rompe niente'
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 500
  Open-Menu 1
  Panel-Click 'Apri'; Start-Sleep -Milliseconds 500
  Check 'la tendina si chiude dopo la scelta' (-not (Panel-Open))
  Check 'la finestra A e ancora li' ([MD]::IsWindow([IntPtr]$ids.A))
  Check 'indice o percorso inventati sono rifiutati' (((Inv 'menubar_choose' ',{index:99,path:[0]}') -eq $false) -and ((Inv 'menubar_choose' ',{index:1,path:[77]}') -eq $false) -and ((Inv 'menubar_choose' ',{index:1,path:[]}') -eq $false))
  Check 'una voce disattivata non si sceglie' ((Inv 'menubar_choose' ',{index:1,path:[3]}') -eq $false)
  Check 'un sottomenu non si sceglie' ((Inv 'menubar_choose' ',{index:1,path:[2]}') -eq $false)

  Write-Host 'G. menu pigri (UI Automation): le voci si leggono all apertura'
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 500
  Check 'la riga e di B' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ($r.read -eq $true) } 6)
  $t0 = Get-Date
  Open-Menu 1
  $ms = [int](((Get-Date) - $t0).TotalMilliseconds)
  $items = Panel-Items
  Check 'File mostra Prima voce e Seconda voce' (($items -join '|') -eq 'Prima voce|Seconda voce') (($items -join '|') + " in $ms ms")
  Check 'la finestra B e ancora viva' ([MD]::IsWindow([IntPtr]$ids.B))
  Close-Panel

  Write-Host 'H. menu Finestra (comandi di EosOS) sulla console di prova'
  Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 500
  Check 'la riga e di C' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.C) -and ($r.read -eq $true) } 6)
  Open-Menu 1
  Check 'Finestra ha Riduci a icona, Ingrandisci, Chiudi' (((Panel-Items) -join '|') -eq 'Riduci a icona|Ingrandisci|Chiudi') ((Panel-Items) -join '|')
  Panel-Click 'Ingrandisci'; Start-Sleep -Milliseconds 800
  Check 'Ingrandisci ingrandisce la console' ([MD]::IsZoomed([IntPtr]$ids.C))
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 300; Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 800
  Open-Menu 1
  Check 'ora la voce e Ripristina' (((Panel-Items) -join '|') -eq 'Riduci a icona|Ripristina|Chiudi') ((Panel-Items) -join '|')
  Panel-Click 'Ripristina'; Start-Sleep -Milliseconds 800
  Check 'Ripristina la riporta normale' (-not [MD]::IsZoomed([IntPtr]$ids.C))
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 300; Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 800
  Open-Menu 1
  Panel-Click 'Riduci a icona'; Start-Sleep -Milliseconds 800
  Check 'Riduci a icona riduce la console' ([MD]::IsIconic([IntPtr]$ids.C))
  [void][MD]::SetForegroundWindow([IntPtr]$ids.A)
  Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 800
  Open-Menu 1
  Panel-Click 'Chiudi'; Start-Sleep -Milliseconds 1500
  Check 'Chiudi chiude la console' (-not [MD]::IsWindow([IntPtr]$ids.C))

  Write-Host 'I. Esci da ... chiude solo l app di prova'
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 500
  Check 'la riga e di A' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) } 6)
  Open-Menu 0
  $it = @(Panel-Items)
  Check 'il menu col nome dell app ha solo Esci da ...' (($it.Count -eq 1) -and ($it[0] -like 'Esci da *')) ($it -join '|')
  Panel-Click $it[0]; Start-Sleep -Milliseconds 1500
  Check 'la finestra A si chiude' (-not [MD]::IsWindow([IntPtr]$ids.A))
  Check 'la finestra B (altra app di prova) e ancora viva' ([MD]::IsWindow([IntPtr]$ids.B))
}
finally {
  Remove-Item $mcPath -ErrorAction SilentlyContinue
  if ($null -ne $mcBackup) { [System.IO.File]::WriteAllBytes($mcPath, $mcBackup) }
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menudrop-*' -or $_.CommandLine -like '*EosOS-Tendina-C*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Remove-Item $formA, $formB -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
