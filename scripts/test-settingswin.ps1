# Verifica S2 delle impostazioni: la finestra "Impostazioni di EosOS". Si apre dal Launcher (azione), sta al centro
# del monitor, mostra le sezioni, gli interruttori e le liste agiscono sui depositi veri, si aggiorna quando
# la stessa cosa cambia da altrove, la ricerca funziona, si chiude con Esc e con il pulsante. I quattro file
# delle impostazioni dell'utente vengono salvati e RIPRISTINATI alla fine. I clic sono solo dentro la finestra
# delle impostazioni di EosOS (via CDP); nessuna finestra di altri toccata; l'unico tasto vero e' Esc, e solo
# dopo aver verificato che la finestra delle impostazioni sia in primo piano.
#
# Uso: powershell -File scripts\test-settingswin.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class SW {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Win([string]$expr) { Js 'settings' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Is-Open { [SW]::Find('EosOS Impostazioni') -ne 0 }
function Labels { @(Win "Array.from(document.querySelectorAll('.settings-label')).map(function(e){return e.childNodes[0].textContent})" | ForEach-Object { $_ }) }
function Navs { @(Win "Array.from(document.querySelectorAll('.settings-nav')).map(function(e){return e.textContent})" | ForEach-Object { $_ }) }
function Heading { Win "document.querySelector('.settings-h').textContent" }
function Click-Nav([string]$name) { Win "(Array.from(document.querySelectorAll('.settings-nav')).find(function(e){return e.textContent==='$name'})||{click:function(){}}).click()" | Out-Null }
function Row-Js([string]$label) { "Array.from(document.querySelectorAll('.settings-row')).find(function(r){return r.querySelector('.settings-label').childNodes[0].textContent==='$label'})" }
function Switch-On([string]$label) { Win ((Row-Js $label) + ".querySelector('[role=switch]').getAttribute('aria-checked')") }
function Click-Switch([string]$label) { Win ((Row-Js $label) + ".querySelector('[role=switch]').click()") | Out-Null }
function Val([string]$key) { (@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq $key }).value }
function Type-In([string]$selector, [string]$text) { Win "(function(){var el=document.querySelector('$selector');var set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;set.call(el,'$text');el.dispatchEvent(new Event('input',{bubbles:true}));return 1})()" | Out-Null }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. apertura dal Launcher'
  Check 'all inizio la finestra non si vede' (-not (Is-Open))
  $r = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:eos-settings'})"
  Check 'l azione del Launcher e accettata' ($r -eq 'done')
  Check 'la finestra compare' (Wait-Until { Is-Open } 5)
  $h = [SW]::Find('EosOS Impostazioni'); $rect = [SW]::Rect($h)
  $scr = [System.Windows.Forms.Screen]::AllScreens | Where-Object { $_.Bounds.Left -le $rect[0] -and $rect[0] -lt $_.Bounds.Right } | Select-Object -First 1
  $cx = ($rect[0] + $rect[2]) / 2; $cy = ($rect[1] + $rect[3]) / 2
  Check 'e al centro di un monitor' (([math]::Abs($cx - ($scr.Bounds.Left + $scr.Bounds.Width / 2)) -le 2) -and ([math]::Abs($cy - ($scr.Bounds.Top + $scr.Bounds.Height / 2)) -le 2)) ("finestra " + ($rect -join ','))
  Check 'misura 780 x 560 a questa scala' ((($rect[2] - $rect[0]) -eq 780) -and (($rect[3] - $rect[1]) -eq 560)) ((($rect[2] - $rect[0]).ToString()) + 'x' + (($rect[3] - $rect[1]).ToString()))
  Check 'ha il primo piano' (Wait-Until { [SW]::GetForegroundWindow() -eq [IntPtr]$h } 3)

  Write-Host 'B. contenuto'
  Check 'la barra laterale ha le sette sezioni' ((Navs) -join '|' -eq 'Generale|Aspetto|Dock|Barra dei menu|Launcher|Mission Control|Notifiche e calendario') ((Navs) -join '|')
  Check 'si parte dalla prima sezione, Generale' ((Heading) -eq 'Generale')
  Click-Nav 'Aspetto'; Start-Sleep -Milliseconds 400
  Check 'Aspetto mostra sfocatura e livello di vetro' ((Labels) -join '|' -eq 'Sfocatura dello sfondo|Livello di vetro') ((Labels) -join '|')
  Check 'ogni riga ha etichetta e aiuto' ((Win "Array.from(document.querySelectorAll('.settings-row')).every(function(r){return r.querySelector('.settings-help').textContent.length>5})") -eq $true)

  Write-Host 'C. interruttori e scelte agiscono sui depositi'
  $blur0 = Val 'dock.blur'
  Click-Switch 'Sfocatura dello sfondo'
  Check 'cliccando l interruttore la sfocatura cambia davvero' (Wait-Until { (Val 'dock.blur') -eq (-not $blur0) } 4)
  Check 'e l interruttore lo mostra' (Wait-Until { (Switch-On 'Sfocatura dello sfondo') -eq (-not $blur0).ToString().ToLower() } 4)
  Click-Switch 'Sfocatura dello sfondo'
  Check 'un altro clic la rimette com era' (Wait-Until { (Val 'dock.blur') -eq $blur0 } 4)
  Win "(function(){var s=document.querySelector('select');var set=Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype,'value').set;set.call(s,'solid');s.dispatchEvent(new Event('change',{bubbles:true}));return 1})()" | Out-Null
  Check 'scegliendo Vetro opaco il livello cambia davvero' (Wait-Until { (Val 'dock.glass') -eq 'solid' } 4)
  Win "(function(){var s=document.querySelector('select');var set=Object.getOwnPropertyDescriptor(HTMLSelectElement.prototype,'value').set;set.call(s,'standard');s.dispatchEvent(new Event('change',{bubbles:true}));return 1})()" | Out-Null
  Check 'e torna standard' (Wait-Until { (Val 'dock.glass') -eq 'standard' } 4)

  Write-Host 'D. si aggiorna quando la stessa cosa cambia da altrove'
  Click-Nav 'Dock'; Start-Sleep -Milliseconds 400
  Check 'la sezione Dock ha nascondi, riserva spazio e app fissate' ((Labels) -join '|' -eq 'Dimensione del Dock|Nascondi automaticamente|Riserva spazio alle finestre|App fissate|Pile') ((Labels) -join '|')
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.reserve',value:false})")
  Check 'un cambio fatto fuori dalla finestra compare nell interruttore' (Wait-Until { (Switch-On 'Riserva spazio alle finestre') -eq 'false' } 4)
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.reserve',value:true})")
  Check 'e anche al ritorno' (Wait-Until { (Switch-On 'Riserva spazio alle finestre') -eq 'true' } 4)

  Write-Host 'E. altre sezioni e lista delle app escluse'
  Click-Nav 'Barra dei menu'; Start-Sleep -Milliseconds 400
  Check 'la sezione cambia' ((Heading) -eq 'Barra dei menu')
  Check 'ci sono indicatori, menu globale e app escluse' (((Labels) -join '|') -eq 'Batteria|Percentuale della batteria|Rete|Volume|Menu globale|App escluse dal menu globale') ((Labels) -join '|')
  Type-In '.names-add input' 'AppDiProva'
  Win "document.querySelector('.names-add button').click()" | Out-Null
  Check 'aggiungendo un nome entra nell elenco vero' (Wait-Until { @(Val 'menu.excluded') -contains 'AppDiProva' } 4)
  Check 'e compare tra le etichette' (Wait-Until { (Win "document.querySelectorAll('.names-item').length") -eq 1 } 4)
  Win "document.querySelector('.names-item button').click()" | Out-Null
  Check 'toglierlo lo toglie davvero' (Wait-Until { @(Val 'menu.excluded').Count -eq 0 } 4)
  Click-Nav 'Mission Control'; Start-Sleep -Milliseconds 400
  Check 'Mission Control mostra la scorciatoia attuale con il pulsante Cambia' (((Win "document.querySelector('.hotkey-keys').textContent") -eq 'Ctrl+Alt+Up') -and ((Win "document.querySelector('.hotkey .settings-button').textContent") -eq 'Cambia'))
  Click-Nav 'Launcher'; Start-Sleep -Milliseconds 400
  Check 'Launcher ha la scelta della scorciatoia con tre opzioni' ((Win "document.querySelector('select').options.length") -eq 3)

  Write-Host 'F. ricerca'
  Type-In '.settings-search' 'batteria'; Start-Sleep -Milliseconds 600
  Check 'il titolo diventa Risultati per' ((Heading) -like 'Risultati per*batteria*') (Heading)
  Check 'il primo risultato e Batteria, con la sua sezione' ((Win "document.querySelector('.settings-label').textContent") -like 'Batteria*Barra dei menu*')
  Check 'nessuna sezione appare selezionata durante la ricerca' ((Win "document.querySelectorAll('.settings-nav-on').length") -eq 0)
  Type-In '.settings-search' 'zzzz'; Start-Sleep -Milliseconds 500
  Check 'una ricerca senza risultati lo dice' ((Win "document.querySelector('.settings-empty').textContent") -like 'Nessuna*')
  Win "window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape'}))" | Out-Null; Start-Sleep -Milliseconds 400
  Check 'Esc svuota prima la ricerca e non chiude la finestra' ((Is-Open) -and ((Win "document.querySelector('.settings-search').value") -eq ''))

  Write-Host 'G. chiusura e riapertura a una sezione'
  Win "document.querySelector('.settings-close').click()" | Out-Null
  Check 'il pulsante chiude (nasconde) la finestra' (Wait-Until { -not (Is-Open) } 4)
  Check 'EosOS e ancora in esecuzione' (@(Get-Process eosos -ErrorAction SilentlyContinue).Count -ge 1)
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'launcher'})")
  Check 'si riapre' (Wait-Until { Is-Open } 4)
  Check 'direttamente alla sezione Launcher' (Wait-Until { (Heading) -eq 'Launcher' } 4)
  $h = [SW]::Find('EosOS Impostazioni')
  if (Wait-Until { [SW]::GetForegroundWindow() -eq [IntPtr]$h } 3) {
    [SW]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [SW]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
    Check 'Esc con la tastiera vera la chiude' (Wait-Until { -not (Is-Open) } 4)
  } else { Check 'la finestra ha il primo piano per provare Esc' $false }

  Write-Host 'H. Mission Control non la tratta come una finestra qualunque'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:null})"); Start-Sleep -Milliseconds 800
  $titles = @(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('mc_windows').then(function(m){return m.flatMap(function(x){return x.windows.map(function(w){return w.title})})})" | ForEach-Object { $_ })
  Check 'tra le finestre di Mission Control non compare' (-not ($titles -contains 'EosOS Impostazioni')) ("finestre viste " + $titles.Count)
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_close')")
}
finally {
  Cleanup-Eos
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
