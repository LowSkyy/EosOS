# Verifica il pannello degli indicatori (I3) con una sorgente SIMULATA: il volume del cursore e il
# silenzio scrivono SOLO nel file finto (EOSOS_FAKE_INDICATORS), mai sull'audio vero. Prima e dopo si
# legge il volume vero (sola lettura) per dimostrare che non e cambiato.
# Clic e tasti veri, ma solo dopo aver verificato che sotto il cursore ci sia una finestra di EosOS
# (la barra o il pannello) e che il tasto vada al pannello in primo piano.
#
# Uso: powershell -File scripts\test-panel.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$bytes = [System.IO.File]::ReadAllBytes($exe)
if (-not [System.Text.Encoding]::GetEncoding(28591).GetString($bytes).Contains('EOSOS_FAKE_INDICATORS')) { Write-Host 'Serve una build di prova (feature test-hooks).'; exit 2 }
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class PN {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern bool GetWindowDisplayAffinity(IntPtr h, out uint a);
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void AltSpace() { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function PJs([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Panel([string]$expr) { PJs 'panel' 0 $expr }
function Wait-Until([scriptblock]$cond, [int]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }; $false }
function Write-Fake($json) { [System.IO.File]::WriteAllText($fake, $json, (New-Object System.Text.UTF8Encoding($false))) }
# Rust sostituisce il file in modo atomico, ma una lettura nell'istante esatto puo fallire: si riprova.
function Read-Fake { for ($k = 0; $k -lt 20; $k++) { try { return (Get-Content $fake -Raw -ErrorAction Stop | ConvertFrom-Json) } catch { Start-Sleep -Milliseconds 50 } }; throw 'file simulato illeggibile' }
function Start-Eos([bool]$useFake) {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  if ($useFake) { $env:EOSOS_FAKE_INDICATORS = $fake } else { Remove-Item env:EOSOS_FAKE_INDICATORS -ErrorAction SilentlyContinue }
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_FAKE_INDICATORS -ErrorAction SilentlyContinue
  Start-Sleep 9
}
function Real-Volume { $v = PJs 'bar' $P "window.__TAURI_INTERNALS__.invoke('indicators_get')"; if ($v.volume) { "$($v.volume.percent)/$($v.volume.muted)" } else { 'assente' } }
function PanelWin { $w = [PN]::Titled('EosOS Pannello'); if ($w.Count) { $w[0] } else { [IntPtr]::Zero } }
function PanelOpen { $h = PanelWin; $h -ne [IntPtr]::Zero -and [PN]::IsWindowVisible($h) }
function PanelFocused { $h = PanelWin; $h -ne [IntPtr]::Zero -and [PN]::GetForegroundWindow() -eq $h }
function Press($vk) { if (PanelFocused) { [PN]::Key([byte]$vk) } else { Write-Host '  (tasto saltato: il pannello non e in primo piano)' -ForegroundColor Yellow } }
# La finestra della barra relativa a una pagina: quella il cui angolo coincide con window.screenX/Y della pagina (a scala 100%).
function BarWindowFor([int]$pageIndex) {
  $sx = [int](PJs 'bar' $pageIndex 'window.screenX'); $sy = [int](PJs 'bar' $pageIndex 'window.screenY')
  foreach ($b in [PN]::Titled('EosOS')) { $r = New-Object PN+R; [void][PN]::GetWindowRect($b, [ref]$r); if ($r.L -eq $sx -and $r.T -eq $sy -and [PN]::IsWindowVisible($b)) { return $b } }
  [IntPtr]::Zero
}
# Clic vero al centro di un indicatore (pagina `pageIndex`), solo se sotto il cursore c'e la sua barra.
function Click-Indicator([int]$pageIndex, [string]$key) {
  $bar = BarWindowFor $pageIndex
  if ($bar -eq [IntPtr]::Zero) { return $false }
  $c = PJs 'bar' $pageIndex "(function(){var r=document.querySelector('.indicator[data-key=$key]').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $r = New-Object PN+R; [void][PN]::GetWindowRect($bar, [ref]$r)
  $x = [int]($r.L + $c[0]); $y = [int]($r.T + $c[1])
  [void][PN]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 350
  if ([PN]::Root($x, $y) -ne $bar) { Write-Host '  (clic saltato: sotto il cursore non c e la barra di EosOS)' -ForegroundColor Yellow; return $false }
  [PN]::Click(); Start-Sleep -Milliseconds 900
  $true
}
$fake = Join-Path $env:TEMP ('eosos-fake-panel-' + (Get-Random -Maximum 99999) + '.json')
$state = '{"battery":{"percent":85,"charging":false,"plugged":false,"minutes_left":135},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4},"volume":{"percent":40,"muted":false}}'
$pb = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$P = 0   # pagina della barra del monitor principale (si determina dopo l'avvio)
try {
  Cleanup-Eos; Start-Sleep 1
  Write-Host 'A. volume VERO prima (sola lettura)'
  Start-Eos $false
  $realBefore = Real-Volume
  Write-Host "  volume vero: $realBefore"
  Cleanup-Eos; Start-Sleep 2

  Write-Fake $state
  Start-Eos $true
  Check 'finestra del pannello creata e nascosta' ((PanelWin) -ne [IntPtr]::Zero -and -not (PanelOpen))
  Start-Sleep -Seconds 2
  foreach ($i in 0..3) { $sx = PJs 'bar' $i 'window.screenX'; $sy = PJs 'bar' $i 'window.screenY'; if ($null -ne $sx -and [int]$sx -eq $pb.X -and [int]$sy -eq $pb.Y) { $P = $i } }
  Write-Host "  pagina della barra del monitor principale: $P"

  Write-Host 'B. clic sull indicatore audio apre il pannello sotto di esso'
  [void][PN]::SetCursorPos($pb.X + 5, $pb.Y + 400)
  Check 'indicatori presenti' (Wait-Until { (PJs 'bar' $P "document.querySelectorAll('.indicator').length") -eq 3 })
  $clicked = Click-Indicator $P 'volume'
  Check 'clic eseguito sulla barra giusta' $clicked
  Check 'pannello visibile' (Wait-Until { PanelOpen })
  $pw = PanelWin; $pr = New-Object PN+R; [void][PN]::GetWindowRect($pw, [ref]$pr)
  $bar = BarWindowFor $P; $br = New-Object PN+R; [void][PN]::GetWindowRect($bar, [ref]$br)
  $ic = PJs 'bar' $P "(function(){var r=document.querySelector('.indicator[data-key=volume]').getBoundingClientRect();return [r.left, r.right, r.bottom]})()"
  Write-Host ('  geometria: pannello {0}..{1} x {2}..{3}; indicatore css {4:N0}..{5:N0}; barra L={6}; schermo principale {7}..{8} x {9}..{10}' -f $pr.L, $pr.Rt, $pr.T, $pr.B, $ic[0], $ic[1], $br.L, $pb.X, $pb.Right, $pb.Y, $pb.Bottom)
  Check 'larghezza 300 px' (($pr.Rt - $pr.L) -eq 300) ("w=" + ($pr.Rt - $pr.L))
  Check 'subito sotto l indicatore (6 px)' ([Math]::Abs($pr.T - ($br.T + $ic[2] + 6)) -le 2) ("T=$($pr.T), atteso " + ($br.T + $ic[2] + 6))
  $icx = $br.L + ($ic[0] + $ic[1]) / 2
  Check 'centrato sull indicatore (o contenuto nello schermo)' (([Math]::Abs((($pr.L + $pr.Rt) / 2) - $icx) -le 2) -or ($pr.Rt -ge $pb.Right - 10))
  Check 'tutto dentro il monitor' ($pr.L -ge $pb.X -and $pr.Rt -le $pb.Right -and $pr.B -le $pb.Bottom)
  Check 'in primo piano' (Wait-Until { PanelFocused })
  Check 'il cursore e a fuoco sul controllo del volume' ((Panel "document.activeElement.className") -eq 'panel-slider')
  Check 'mostra il volume 40%' (((Panel "document.querySelector('.panel-slider').value") -eq '40') -and ((Panel "document.querySelector('.panel-percent').textContent") -eq '40%'))
  [uint32]$aff = 0; [void][PN]::GetWindowDisplayAffinity($pw, [ref]$aff)
  Check 'finestra esclusa dalla cattura (sfocatura)' ($aff -eq 0x11)
  Check 'sfondo sfocato attivo' (Wait-Until { (Panel "document.documentElement.getAttribute('data-blur')") -eq 'on' })
  $h = [double](Panel "document.querySelector('.panel-body').getBoundingClientRect().height")
  Check 'la finestra segue l altezza del contenuto' ([Math]::Abs(($pr.B - $pr.T) - [Math]::Ceiling($h)) -le 2) ("finestra " + ($pr.B - $pr.T) + ", contenuto $h")

  if ($env:PANEL_SHOT) { $env:CDP_VIEW = 'panel'; & node $cdp shot ($env:PANEL_SHOT + '-volume.png') | Out-Null }

  Write-Host 'C. cursore con la tastiera: scrive solo nel file simulato'
  foreach ($i in 1..5) { Press 0x27; Start-Sleep -Milliseconds 70 }   # freccia destra x5
  Check 'il cursore e a 45' ((Panel "document.querySelector('.panel-slider').value") -eq '45')
  Check 'il file simulato riceve 45' (Wait-Until { (Read-Fake).volume.percent -eq 45 }) ("file: " + (Read-Fake).volume.percent)
  Check 'gli altri dati del file restano' (((Read-Fake).battery.percent -eq 85) -and ((Read-Fake).network.name -eq 'Casa'))
  Press 0x24; Start-Sleep -Milliseconds 300     # Home
  Check 'Home: 0%' (Wait-Until { (Read-Fake).volume.percent -eq 0 })
  Press 0x23; Start-Sleep -Milliseconds 300     # End
  Check 'Fine: 100%' (Wait-Until { (Read-Fake).volume.percent -eq 100 })
  Check 'la barra segue (dopo il controllo periodico)' (Wait-Until { (PJs 'bar' $P "document.querySelector('.indicator[data-key=volume]').title") -eq 'Volume 100%' } 5)
  foreach ($i in 1..30) { Press 0x25 }   # freccia sinistra x30 -> 70
  Check 'molti passi di fila: arriva il valore finale (70)' (Wait-Until { (Read-Fake).volume.percent -eq 70 } 5) ("file: " + (Read-Fake).volume.percent)

  Write-Host 'D. pulsante del silenzio (clic vero sul pannello)'
  $mc = Panel "(function(){var r=document.querySelector('.panel-mute').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $mx = [int]($pr.L + $mc[0]); $my = [int]($pr.T + $mc[1])
  [void][PN]::SetCursorPos($mx, $my); Start-Sleep -Milliseconds 350
  $underPanel = ([PN]::Root($mx, $my) -eq $pw)
  Check 'sotto il cursore c e il pannello (si clicca solo se e vero)' $underPanel
  if ($underPanel) { [PN]::Click(); Start-Sleep -Milliseconds 700 }
  Check 'il file simulato ha il silenzio' (Wait-Until { (Read-Fake).volume.muted -eq $true })
  Check 'il pannello mostra il silenzio' ((Panel "document.querySelector('.panel-percent').textContent") -ne '70%' -and (Panel "document.querySelector('.panel-mute').getAttribute('aria-pressed')") -eq 'true')
  Check 'la barra mostra audio disattivato' (Wait-Until { (PJs 'bar' $P "document.querySelector('.indicator[data-key=volume]').title") -eq 'Audio disattivato' } 5)
  [void](Panel "(document.querySelector('.panel-slider').focus(),1)"); Start-Sleep -Milliseconds 300   # dopo il clic il fuoco e sul pulsante: si rimette sul cursore
  Press 0x27; Start-Sleep -Milliseconds 400   # muovere il cursore con l'audio spento lo riaccende
  Check 'muovere il cursore riattiva l audio' (Wait-Until { (Read-Fake).volume.muted -eq $false })
  Check 'e il volume sale di un passo' (Wait-Until { (Read-Fake).volume.percent -eq 71 }) ("file: " + (Read-Fake).volume.percent)

  Write-Host 'E. chiusura: Esc, secondo clic, perdita di focus, cambio di indicatore'
  Press 0x1B; Start-Sleep -Milliseconds 600
  Check 'Esc chiude' (-not (PanelOpen))
  [void](Click-Indicator $P 'volume'); Check 'riaperto' (Wait-Until { PanelOpen })
  [void](Click-Indicator $P 'volume'); Start-Sleep -Milliseconds 800
  Check 'un secondo clic sullo stesso indicatore lo chiude (e non lo riapre)' (-not (PanelOpen))
  [void](Click-Indicator $P 'volume'); Check 'riaperto' (Wait-Until { PanelOpen })
  [void](Click-Indicator $P 'network'); Start-Sleep -Milliseconds 900
  Check 'cliccare un altro indicatore cambia il contenuto' ((PanelOpen) -and ((Panel "document.querySelector('.panel-title').textContent") -eq 'Rete'))
  $rows = Panel "Array.from(document.querySelectorAll('.panel-row')).map(function(e){return e.textContent}).join('|')"
  Check 'rete: nome, tipo, stato e segnale' (($rows -like '*Casa*') -and ($rows -like '*Wi-Fi*') -and ($rows -like '*Connesso a Internet*') -and ($rows -like '*4 su 5*')) $rows
  if ($env:PANEL_SHOT) { $env:CDP_VIEW = 'panel'; & node $cdp shot ($env:PANEL_SHOT + '-network.png') | Out-Null }
  Check 'rete: pulsante per le Impostazioni' ((Panel "document.querySelector('.panel-link').textContent") -like 'Impostazioni di rete*')
  [void](Click-Indicator $P 'battery'); Start-Sleep -Milliseconds 900
  Check 'batteria: titolo e percentuale grande' (((Panel "document.querySelector('.panel-title').textContent") -eq 'Batteria') -and ((Panel "document.querySelector('.panel-big').textContent") -eq '85%'))
  $rows = Panel "Array.from(document.querySelectorAll('.panel-row')).map(function(e){return e.textContent}).join('|')"
  if ($env:PANEL_SHOT) { $env:CDP_VIEW = 'panel'; & node $cdp shot ($env:PANEL_SHOT + '-battery.png') | Out-Null }
  Check 'batteria: stato e tempo rimanente' (($rows -like '*A batteria*') -and ($rows -like '*2 h 15 min*')) $rows
  # Perdita di focus: si apre il Launcher (finestra di EosOS) e il pannello deve chiudersi.
  [PN]::AltSpace(); Start-Sleep -Milliseconds 1200
  Check 'aprendo un altra finestra il pannello si chiude da solo' (-not (PanelOpen))
  $lw = [PN]::Titled('EosOS Launcher'); if ($lw.Count -and [PN]::GetForegroundWindow() -eq $lw[0]) { [PN]::Key(0x1B) }
  Start-Sleep -Milliseconds 600

  Write-Host 'E2. l indicatore sparisce mentre il pannello e aperto (per esempio si scollega la cuffia)'
  [void](Click-Indicator $P 'volume'); Check 'pannello audio aperto' (Wait-Until { PanelOpen })
  Write-Fake '{"battery":{"percent":85},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4}}'
  Check 'il pannello si chiude da solo (niente finestra vuota)' (Wait-Until { -not (PanelOpen) } 5)
  Write-Fake $state
  Check 'tornano i tre indicatori' (Wait-Until { (PJs 'bar' $P "document.querySelectorAll('.indicator').length") -eq 3 })

  Write-Host 'F. senza indicatore non si apre; un secondo monitor'
  Write-Fake '{"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4}}'
  Check 'senza audio resta solo la rete' (Wait-Until { (PJs 'bar' $P "document.querySelectorAll('.indicator').length") -eq 1 })
  Write-Fake $state
  Check 'tornano i tre indicatori' (Wait-Until { (PJs 'bar' $P "document.querySelectorAll('.indicator').length") -eq 3 })
  $pages = 0; foreach ($i in 0..3) { if ($null -ne (PJs 'bar' $i 'window.screenX')) { $pages++ } }
  if ($pages -ge 2) {
    $sec = $null; foreach ($i in 0..($pages - 1)) { if ($i -ne $P) { $sec = $i } }
    if ($null -ne $sec -and (Click-Indicator $sec 'network')) {
      Check 'pannello aperto' (Wait-Until { PanelOpen })
      $pw = PanelWin; $pr = New-Object PN+R; [void][PN]::GetWindowRect($pw, [ref]$pr)
      $sb = BarWindowFor $sec; $sbr = New-Object PN+R; [void][PN]::GetWindowRect($sb, [ref]$sbr)
      Check 'sul monitor della barra cliccata (non quello principale)' ($pr.L -ge $sbr.L -and $pr.Rt -le $sbr.Rt -and $pr.T -ge $sbr.T) ("pannello $($pr.L)..$($pr.Rt), barra $($sbr.L)..$($sbr.Rt)")
      Press 0x1B; Start-Sleep -Milliseconds 500
    }
  } else { Write-Host '  (un solo monitor)' }

  Write-Host 'G. il volume VERO non e cambiato'
  Cleanup-Eos; Start-Sleep 2
  Start-Eos $false
  $realAfter = Real-Volume
  Check "volume vero prima ($realBefore) = dopo ($realAfter)" ($realBefore -eq $realAfter)
}
finally {
  Cleanup-Eos
  Remove-Item $fake -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
