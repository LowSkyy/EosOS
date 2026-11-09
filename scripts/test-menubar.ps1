# Verifica G3 del menu globale: la barra mostra il nome dell'app in primo piano e la fila dei suoi menu. Finestre di
# prova NUOVE: A con menu classico (HMENU), B con barra MenuStrip (UI Automation), C console classica (senza menu).
# Nessun clic sulla barra, nessun tasto, nessuna voce invocata; la finestra in primo piano si cambia solo fra le mie.
#
# Uso: powershell -File scripts\test-menubar.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MB {
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
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Row { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('menubar_current')" }
function BarTitles { @(Js 'bar' "Array.from(document.querySelectorAll('.app-menu')).map(function(e){return e.textContent})" | ForEach-Object { $_ }) }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][MB]::SetForegroundWindow($h)
    if (Wait-Until { [MB]::GetForegroundWindow() -eq $h } 1) { return }
    [MB]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [MB]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Titles($r) { @($r.menus | ForEach-Object { $_.title }) }

$formA = Join-Path $env:TEMP 'eosos-menubar-a.ps1'; $formB = Join-Path $env:TEMP 'eosos-menubar-b.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Barra-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
foreach ($n in '&File', 'Modifica', 'Visualizza', 'Aiuto') { $i = New-Object System.Windows.Forms.MenuItem $n; [void]$i.MenuItems.Add('&Apri' + [char]9 + 'Ctrl+O'); [void]$i.MenuItems.Add('-'); [void]$i.MenuItems.Add('Esci'); [void]$m.MenuItems.Add($i) }
$f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8
Set-Content $formB @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Barra-B'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MenuStrip
foreach ($n in 'File', 'Modifica', 'Aiuto') { $i = New-Object System.Windows.Forms.ToolStripMenuItem $n; [void]$i.DropDownItems.Add('Voce'); [void]$m.Items.Add($i) }
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
  Start-Process 'conhost.exe' -ArgumentList 'powershell.exe -NoProfile -NoExit -Command "$Host.UI.RawUI.WindowTitle=''EosOS-Barra-C''"' | Out-Null
  $ids = @{}
  Wait-Until { $ids.A = [MB]::Find('EosOS-Barra-A'); $ids.B = [MB]::Find('EosOS-Barra-B'); $ids.C = [MB]::Find('EosOS-Barra-C'); ($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0) } 20 | Out-Null
  Check 'finestre di prova create' (($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0))
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; if ($env:MB_ERR) { $env:EOSOS_DEBUG_FOREGROUND = '1'; Start-Process $exe -RedirectStandardError $env:MB_ERR | Out-Null } else { Start-Process $exe | Out-Null }; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 10
  Start-Sleep 2

  Write-Host 'A. finestra con menu classico (HMENU)'
  Focus ([IntPtr]$ids.C); Start-Sleep -Milliseconds 800   # il primo piano deve CAMBIARE perche' l'hook di sistema emetta l'evento
  Focus ([IntPtr]$ids.A)
  Check 'la riga e della finestra A e arriva LETTA' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) } 6)
  $r = Row; $t = Titles $r
  Check 'nome dell app, File, Modifica, Visualizza, Aiuto, Finestra' (($t.Count -eq 6) -and ($t[1] -eq 'File') -and ($t[2] -eq 'Modifica') -and ($t[3] -eq 'Visualizza') -and ($t[4] -eq 'Aiuto') -and ($t[5] -eq 'Finestra')) ($t -join ' | ')
  Check 'l acceleratore & del titolo e stato tolto' ($t -notcontains '&File')
  Check 'il primo menu e in grassetto, gli altri no' (($r.menus[0].bold -eq $true) -and ($r.menus[1].bold -eq $false))
  $fm = $r.menus[1]
  Check 'File ha 3 voci: Apri con scorciatoia Ctrl+O, separatore, Esci' (($fm.items.Count -eq 3) -and ($fm.items[0].title -eq 'Apri') -and ($fm.items[0].shortcut -eq 'Ctrl+O') -and ($fm.items[1].separator -eq $true) -and ($fm.items[2].title -eq 'Esci'))
  Check 'la voce Apri indica la sua posizione originale (menu 0, voce 0)' ((($fm.items[0].action.value) -join ',') -eq '0,0')
  Check 'la barra mostra gli stessi titoli' ((BarTitles) -join '|' -eq ($t -join '|')) ((BarTitles) -join ' | ')

  Write-Host 'B. finestra con MenuStrip (UI Automation): menu pigri'
  Focus ([IntPtr]$ids.B)
  Check 'arriva letta' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.B) -and ($r.read -eq $true) } 6)
  $r = Row; $t = Titles $r
  Check 'File, Modifica, Aiuto fra il nome e Finestra' (($t.Count -eq 5) -and ($t[1] -eq 'File') -and ($t[2] -eq 'Modifica') -and ($t[3] -eq 'Aiuto') -and ($t[4] -eq 'Finestra')) ($t -join ' | ')
  Check 'sono pigri (voci da leggere all apertura)' (($r.menus[1].lazy -eq $true) -and ($r.menus[1].items.Count -eq 0))

  Write-Host 'C. console senza menu: solo nome e Finestra'
  Focus ([IntPtr]$ids.C)
  Check 'arriva letta' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.C) -and ($r.read -eq $true) } 6)
  $t = Titles (Row)
  Check 'restano solo il nome e Finestra' (($t.Count -eq 2) -and ($t[1] -eq 'Finestra')) ($t -join ' | ')
  Check 'il menu Finestra ha Riduci a icona, Ingrandisci, Chiudi' ((Row).menus[1].items.Count -eq 4)

  Write-Host 'D. cambi rapidi: vince l ultima finestra, nessuna riga vecchia sovrascrive'
  Focus ([IntPtr]$ids.A); Focus ([IntPtr]$ids.B); Focus ([IntPtr]$ids.C)
  Start-Sleep -Milliseconds 2500
  $r = Row
  Check 'la riga finale e quella di C (console)' ($r.hwnd -eq [int64]$ids.C) ("hwnd " + $r.hwnd)
  Check 'e la barra mostra nome + Finestra' ((BarTitles).Count -eq 2)

  Write-Host 'E. cache: tornare su A subito dopo mostra la riga gia letta'
  Focus ([IntPtr]$ids.A)
  $t0 = Get-Date
  Check 'letta entro 600 ms' (Wait-Until { $r = Row; ($r.hwnd -eq [int64]$ids.A) -and ($r.read -eq $true) } 0.6) ("in " + [int](((Get-Date) - $t0).TotalMilliseconds) + " ms")
  Check 'con tutti i menu' ((Titles (Row)).Count -eq 6)
}
finally {
  Remove-Item $mcPath -ErrorAction SilentlyContinue
  if ($null -ne $mcBackup) { [System.IO.File]::WriteAllBytes($mcPath, $mcBackup) }
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menubar-*' -or $_.CommandLine -like '*EosOS-Barra-C*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Remove-Item $formA, $formB -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
