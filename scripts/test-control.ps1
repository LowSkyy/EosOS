# Centro di controllo: il pulsante nella barra apre il pannello con gli interruttori. Wi-Fi e Bluetooth sono FINTI (EOSOS_TEST_RADIOS=fake: stato in
# memoria, mai le radio vere: spegnere il Wi-Fi vero taglierebbe la rete dell'utente); Mantieni sveglio si accende e si spegne davvero (e si
# controlla che si spenga); Non disturbare e' l'impostazione notif.dnd (salvata e ripristinata). Il volume NON si tocca. Una finestra diversa
# dal pannello non puo' cambiare gli interruttori (controllo accessi). La lettura delle radio vere si prova solo in lettura.
#
# Uso: powershell -File scripts\test-control.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class CT { public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 public static bool Visible(string part){ bool f=false; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString().Contains(part)){f=true;return false;} return true;},IntPtr.Zero); return f; } }
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Pn([string]$e) { Js 'panel' $e }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Open-Control { Js 'bar' "document.querySelector('.control-btn').click()" | Out-Null; Wait-Until { ([CT]::Visible('EosOS Pannello')) -and ((Pn "document.querySelectorAll('.control-tile').length") -ge 3) } 6 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not ([CT]::Visible('EosOS Pannello')) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }
function Tile([string]$id) { Pn "(function(){var t=document.querySelector('.control-tile[data-id=\'$id\']');return t?t.getAttribute('data-on'):null})()" }
function Click-Tile([string]$id) { Pn "document.querySelector('.control-tile[data-id=\'$id\']').click()" | Out-Null; Start-Sleep -Milliseconds 700 }
function State { Pn "window.__TAURI_INTERNALS__.invoke('control_state')" }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOSOS_TEST_RADIOS = 'fake'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_TEST_RADIOS; Start-Sleep 12

  Write-Host 'A. il pulsante e il pannello'
  Check 'c e il pulsante nella barra, a destra' ((Js 'bar' "document.querySelector('.menubar-right .control-btn')!==null") -eq $true)
  Check 'ha un nome per i lettori di schermo' ((Js 'bar' "document.querySelector('.control-btn').getAttribute('aria-label')") -eq 'Centro di controllo')
  Check 'un clic apre il pannello' (Open-Control)
  $ids = Pn "Array.from(document.querySelectorAll('.control-tile')).map(function(t){return t.getAttribute('data-id')}).join(',')"
  Check 'gli interruttori sono Wi-Fi, Bluetooth, Non disturbare, Mantieni sveglio' ($ids -eq 'wifi,bluetooth,dnd,awake') $ids
  Check 'le radio finte partono: Wi-Fi acceso, Bluetooth spento' (((Tile 'wifi') -eq 'true') -and ((Tile 'bluetooth') -eq 'false'))
  Check 'Mantieni sveglio parte spento' ((Tile 'awake') -eq 'false')
  Check 'ogni interruttore e un pulsante con aria-pressed' ((Pn "Array.from(document.querySelectorAll('.control-tile')).every(function(t){return t.hasAttribute('aria-pressed')})") -eq $true)
  Check 'ci sono i due pulsanti di cattura dello schermo' ((Pn "Array.from(document.querySelectorAll('.control-cap')).map(function(b){return b.textContent.trim()}).join('|')") -like 'Schermo intero|Selezione*')
  Check 'il primo controllo ha il focus' (Wait-Until { (Pn "document.activeElement && document.activeElement.classList.contains('control-tile')") -eq $true } 3)

  Write-Host 'B. Wi-Fi e Bluetooth (finti)'
  Click-Tile 'wifi'
  Check 'spegnere il Wi-Fi: la tessera si spegne' ((Tile 'wifi') -eq 'false')
  Check 'e Rust lo conferma' ((State).wifi -eq $false)
  Click-Tile 'bluetooth'
  Check 'accendere il Bluetooth: la tessera si accende' ((Tile 'bluetooth') -eq 'true')
  Check 'e Rust lo conferma' ((State).bluetooth -eq $true)
  Close-Panel; [void](Open-Control)
  Check 'riaprendo il pannello lo stato e quello vero (non quello di prima)' (((Tile 'wifi') -eq 'false') -and ((Tile 'bluetooth') -eq 'true'))
  Check 'un id inventato e rifiutato' ((Pn "window.__TAURI_INTERNALS__.invoke('control_set',{id:'boh',on:true})") -eq $false)
  Check 'anche Non disturbare non passa da qui' ((Pn "window.__TAURI_INTERNALS__.invoke('control_set',{id:'dnd',on:true})") -eq $false)

  Write-Host 'C. Non disturbare'
  $before = Pn "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(r){return r.filter(function(x){return x.key==='notif.dnd'})[0].value})"
  Click-Tile 'dnd'
  Check 'la tessera cambia' ((Tile 'dnd') -eq $(if ($before -eq $true) { 'false' } else { 'true' }))
  $after = Pn "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(r){return r.filter(function(x){return x.key==='notif.dnd'})[0].value})"
  Check 'e l impostazione notif.dnd e cambiata davvero' ($after -ne $before)
  Click-Tile 'dnd'
  Check 'un secondo clic la riporta com era' ((Tile 'dnd') -eq $(if ($before -eq $true) { 'true' } else { 'false' }))

  Write-Host 'D. Mantieni sveglio'
  Click-Tile 'awake'
  Check 'acceso: la tessera e Rust lo dicono' (((Tile 'awake') -eq 'true') -and ((State).awake -eq $true))
  Click-Tile 'awake'
  Check 'spento: la tessera e Rust lo dicono' (((Tile 'awake') -eq 'false') -and ((State).awake -eq $false))

  Write-Host 'E. chiusura e tastiera'
  Pn "document.querySelector('.control-tile').focus()" | Out-Null
  Pn "document.querySelector('.control').dispatchEvent(new KeyboardEvent('keydown',{key:'ArrowRight',bubbles:true}))" | Out-Null
  Start-Sleep -Milliseconds 300
  Check 'la freccia destra sposta il focus sul controllo dopo' ((Pn "document.activeElement && document.activeElement.getAttribute('data-id')") -eq 'bluetooth')
  Close-Panel
  Check 'il pannello si chiude' (-not ([CT]::Visible('EosOS Pannello')))
  [void](Js 'bar' "document.querySelector('.control-btn').click()"); Start-Sleep 1
  [void](Js 'bar' "document.querySelector('.control-btn').click()"); Start-Sleep 1
  Check 'un secondo clic sul pulsante lo richiude' (-not ([CT]::Visible('EosOS Pannello')))

  Write-Host 'F. controllo accessi'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'; $env:EOSOS_TEST_RADIOS = 'fake'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_TEST_RADIOS; Start-Sleep 12
  Check 'dalla barra control_state non da niente' ($null -eq (Js 'bar' "window.__TAURI_INTERNALS__.invoke('control_state').then(function(r){return r},function(e){return null})"))
  Check 'dalla barra control_set non cambia niente' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('control_set',{id:'awake',on:true}).then(function(r){return r},function(e){return false})") -ne $true)
  Check 'dal Launcher neppure' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('control_set',{id:'awake',on:true}).then(function(r){return r},function(e){return false})") -ne $true)
  Check 'dal pannello si puo leggere' (Wait-Until { $null -ne (Pn "window.__TAURI_INTERNALS__.invoke('control_state')") } 5)

  Write-Host 'G. radio vere: solo lettura'
  Cleanup-Eos; Start-Sleep 1; Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $real = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('control_state')"
  Check 'lo stato vero si legge (Wi-Fi e Bluetooth veri o assenti, mai un errore)' ($null -ne $real) ("wifi=" + $real.wifi + " bluetooth=" + $real.bluetooth + " awake=" + $real.awake)
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT, env:EOSOS_TEST_RADIOS -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
