# Verifica G5 del menu globale: scegliere una voce del menu di un'app dalla tendina di EosOS la ESEGUE nell'app.
# SOLO su app di prova NUOVE create qui: A con menu classico (HMENU) e B con barra MenuStrip (UI Automation).
# Ogni voce, quando viene eseguita, scrive una riga in un file di log temporaneo: il test guarda quel file.
# I clic sono solo sulla tendina di EosOS (via CDP); nessun tasto, nessun clic su finestre di altri.
#
# Uso: powershell -File scripts\test-menuinvoke.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MI {
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
function Inv([string]$cmd, [string]$a = '') { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('$cmd'$a)" }
function Row { Inv 'menubar_current' }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][MI]::SetForegroundWindow($h)
    if (Wait-Until { [MI]::GetForegroundWindow() -eq $h } 1) { return }
    [MI]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [MI]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Panel-Open { [MI]::Find('EosOS Pannello') -ne 0 }
function Panel-Items { @(Js 'panel' "Array.from(document.querySelectorAll('.menu-item')).map(function(e){return e.querySelector('.menu-title').textContent})" | ForEach-Object { $_ }) }
function Bar-Click([int]$i) { Js 'bar' "document.querySelectorAll('.app-menu')[$i].click()" | Out-Null }
function Panel-Click([string]$title) { Js 'panel' "(Array.from(document.querySelectorAll('.menu-item')).find(function(e){return e.querySelector('.menu-title').textContent==='$title'})||{click:function(){}}).click()" | Out-Null }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null }
function Open-Menu([int]$i) { Close-Panel; Bar-Click $i; Wait-Until { (Panel-Open) -and ((Panel-Items).Count -gt 0) } 6 | Out-Null }
function Log-Lines { if (Test-Path $log) { @(Get-Content $log) } else { @() } }
function Log-Has([string]$line, [double]$sec = 4) { Wait-Until { (Log-Lines) -contains $line } $sec }
function Log-Count([string]$line) { @(Log-Lines | Where-Object { $_ -eq $line }).Count }

$log = Join-Path $env:TEMP 'eosos-menuinvoke-log.txt'; Remove-Item $log -ErrorAction SilentlyContinue
$env:EOS_TEST_LOG = $log
$formA = Join-Path $env:TEMP 'eosos-menuinvoke-a.ps1'; $formB = Join-Path $env:TEMP 'eosos-menuinvoke-b.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$log = $env:EOS_TEST_LOG
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Invoca-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
$file = New-Object System.Windows.Forms.MenuItem '&File'
$apri = New-Object System.Windows.Forms.MenuItem '&Apri'; $apri.add_Click({ Add-Content $log 'A:Apri' }.GetNewClosure())
$salva = New-Object System.Windows.Forms.MenuItem 'Salva'; $salva.Enabled = $false; $salva.add_Click({ Add-Content $log 'A:Salva-NON-DEVE-SUCCEDERE' }.GetNewClosure())
$rec = New-Object System.Windows.Forms.MenuItem 'Recenti'
$uno = New-Object System.Windows.Forms.MenuItem 'uno'; $uno.add_Click({ Add-Content $log 'A:uno' }.GetNewClosure()); [void]$rec.MenuItems.Add($uno)
$ren = New-Object System.Windows.Forms.MenuItem 'Rinomina'; $ren.add_Click({ Add-Content $log 'A:Rinomina' }.GetNewClosure())
$tm = New-Object System.Windows.Forms.Timer; $tm.Interval = 200
$tm.add_Tick({ if (Test-Path ($log + '.trigger')) { Remove-Item ($log + '.trigger'); $apri.Text = 'Aprire'; Add-Content $log 'A:rinominato-da-timer' } }.GetNewClosure()); $tm.Start()
foreach ($x in $apri, $salva, $rec, $ren) { [void]$file.MenuItems.Add($x) }
[void]$m.MenuItems.Add($file); $f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8
Set-Content $formB @'
Add-Type -AssemblyName System.Windows.Forms
$log = $env:EOS_TEST_LOG
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Invoca-B'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MenuStrip
$file = New-Object System.Windows.Forms.ToolStripMenuItem 'File'
$p = New-Object System.Windows.Forms.ToolStripMenuItem 'Prima voce'; $p.add_Click({ Add-Content $log 'B:Prima' }.GetNewClosure())
$s = New-Object System.Windows.Forms.ToolStripMenuItem 'Seconda voce'; $s.add_Click({ Add-Content $log 'B:Seconda' }.GetNewClosure())
[void]$file.DropDownItems.Add($p); [void]$file.DropDownItems.Add($s)
$ed = New-Object System.Windows.Forms.ToolStripMenuItem 'Modifica'; [void]$ed.DropDownItems.Add('Altro')
[void]$m.Items.Add($file); [void]$m.Items.Add($ed)
$f.MainMenuStrip = $m; $f.Controls.Add($m); [void]$f.ShowDialog()
'@ -Encoding UTF8
$mcSeed = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = if (Test-Path $mcSeed) { [System.IO.File]::ReadAllBytes($mcSeed) } else { $null }
try {
  # Di base il nome dell'app non si mostra in barra: questa prova conta i menu dal nome, quindi lo si accende nel file (salvato prima e ripristinato dopo).
  New-Item -ItemType Directory -Force (Split-Path $mcSeed) | Out-Null
  [System.IO.File]::WriteAllText($mcSeed, '{"menuShowName": true}', (New-Object System.Text.UTF8Encoding($false)))
  Cleanup-Eos; Start-Sleep 1
  Write-Host 'Preparazione: due app di prova NUOVE che scrivono un log quando una voce viene eseguita'
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formA | Out-Null
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formB | Out-Null
  $ids = @{}
  Wait-Until { $ids.A = [MI]::Find('EosOS-Invoca-A'); $ids.B = [MI]::Find('EosOS-Invoca-B'); ($ids.A -ne 0) -and ($ids.B -ne 0) } 20 | Out-Null
  Check 'app di prova create' (($ids.A -ne 0) -and ($ids.B -ne 0))
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. menu classico (HMENU): WM_COMMAND'
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 600; Focus ([IntPtr]$ids.A)
  Check 'la riga e di A, letta' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) } 6)
  Open-Menu 1
  Check 'File: Apri, Salva, Recenti, Rinomina' (((Panel-Items) -join '|') -eq 'Apri|Salva|Recenti|Rinomina') ((Panel-Items) -join '|')
  Panel-Click 'Apri'
  Check 'scegliere Apri la esegue nell app (riga nel log)' (Log-Has 'A:Apri')
  Check 'la tendina si e chiusa' (-not (Panel-Open))
  Check 'il primo piano e tornato all app' (Wait-Until { [MI]::GetForegroundWindow() -eq [IntPtr]$ids.A } 3)

  Write-Host 'B. sottomenu (percorso di tre livelli)'
  Open-Menu 1
  Panel-Click 'Recenti'; Start-Sleep -Milliseconds 400
  Panel-Click 'uno'
  Check 'scegliere uno dentro Recenti la esegue' (Log-Has 'A:uno')

  Write-Host 'C. voce disattivata: mai eseguita'
  Open-Menu 1
  Check 'il pulsante di Salva e disattivato' ((Js 'panel' "Array.from(document.querySelectorAll('.menu-item')).find(function(e){return e.querySelector('.menu-title').textContent==='Salva'}).disabled") -eq $true)
  Check 'neanche chiamando il comando a mano (percorso 1,1)' ((Inv 'menubar_choose' ',{index:1,path:[1]}') -eq $false)
  Start-Sleep -Milliseconds 600
  Check 'Salva non e mai stata eseguita' (-not ((Log-Lines) -contains 'A:Salva-NON-DEVE-SUCCEDERE'))

  Write-Host 'D. il menu cambia dopo che e stato letto: non si esegue la voce sbagliata'
  Open-Menu 1
  Panel-Click 'Rinomina'
  Check 'una voce qualunque (Rinomina) viene eseguita' (Log-Has 'A:Rinomina')
  Start-Sleep -Milliseconds 800
  Open-Menu 1
  Check 'la tendina mostra Apri' (((Panel-Items) -join '|') -like 'Apri|*') ((Panel-Items) -join '|')
  Set-Content ($log + '.trigger') 'x'   # l app, da sola, rinomina Apri in Aprire mentre la tendina e aperta
  Check 'l app ha rinominato la voce' (Log-Has 'A:rinominato-da-timer' 4)
  $before = Log-Count 'A:Apri'
  Panel-Click 'Apri'
  Start-Sleep -Milliseconds 1200
  Check 'scegliere la vecchia voce NON esegue niente (il titolo non coincide piu)' ((Log-Count 'A:Apri') -eq $before) ("esecuzioni di Apri: " + (Log-Count 'A:Apri'))
  Check 'l app e ancora viva' ([MI]::Title([IntPtr]$ids.A) -eq 'EosOS-Invoca-A')
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 800; Focus ([IntPtr]$ids.A)
  Check 'cambiando finestra la barra rilegge: ora la voce e Aprire' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) -and (@($r.menus[1].items | ForEach-Object { $_.title }) -contains 'Aprire') } 6)
  Open-Menu 1
  Panel-Click 'Aprire'
  Check 'e la voce col nome nuovo si esegue (la riga e stata rinfrescata)' (Wait-Until { (Log-Count 'A:Apri') -eq ($before + 1) } 4)

  Write-Host 'E. barra UI Automation (MenuStrip): Invoke'
  Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 500
  Check 'la riga e di B' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ($r.read -eq $true) } 6)
  Open-Menu 1
  Check 'File: Prima voce, Seconda voce' (((Panel-Items) -join '|') -eq 'Prima voce|Seconda voce') ((Panel-Items) -join '|')
  Panel-Click 'Seconda voce'
  Check 'scegliere Seconda voce la esegue' (Log-Has 'B:Seconda' 6)
  Check 'e non ha eseguito Prima voce' (-not ((Log-Lines) -contains 'B:Prima'))
  Start-Sleep -Milliseconds 600
  Check 'l app B e ancora viva' ([MI]::Title([IntPtr]$ids.B) -eq 'EosOS-Invoca-B')
  Focus ([IntPtr]$ids.A); Start-Sleep -Milliseconds 400; Focus ([IntPtr]$ids.B); Start-Sleep -Milliseconds 500
  Open-Menu 1
  Panel-Click 'Prima voce'
  Check 'e poi Prima voce' (Log-Has 'B:Prima' 6)
}
finally {
  Remove-Item $mcSeed -ErrorAction SilentlyContinue
  if ($null -ne $mcBackup) { [System.IO.File]::WriteAllBytes($mcSeed, $mcBackup) }
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menuinvoke-*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Remove-Item $formA, $formB, $log -ErrorAction SilentlyContinue
  Remove-Item env:EOS_TEST_LOG -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
