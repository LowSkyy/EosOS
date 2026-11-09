# Impostazioni di Windows dentro la finestra delle Impostazioni di EosOS: sezione «Impostazioni di Windows» con l'elenco
# raggruppato e la ricerca; un clic su una voce chiede a Rust di aprire la pagina giusta (con EOSOS_TEST_NO_OPEN non si apre
# niente di vero: si registra l'indirizzo); un id inventato e rifiutato; dalle altre finestre i comandi sono vietati.
# mission.json salvato e ripristinato.
# Uso: powershell -File scripts\test-winsettings.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Win([string]$e) { Js 'settings' $e }
function Type-Search([string]$text) {
  Win "(function(){var set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;var i=document.querySelector('.winset-search');set.call(i,'$text');i.dispatchEvent(new Event('input',{bubbles:true}));return 1})()" | Out-Null
  Start-Sleep -Milliseconds 500
}
function Click-Tile([string]$label) { Win "Array.from(document.querySelectorAll('.winset-tile')).filter(function(b){return b.querySelector('.winset-label').textContent==='$label'})[0].click()" | Out-Null }
function Last-Open { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('winsettings_last_open')" }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOSOS_TEST_NO_OPEN = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'windows'})")
  Start-Sleep -Milliseconds 1500

  Write-Host 'A. la sezione'
  Check 'la sezione Impostazioni di Windows e nella barra laterale (ultima)' ((Win "Array.from(document.querySelectorAll('.settings-nav')).map(function(b){return b.textContent.trim()}).join('|')") -like '*|Impostazioni di Windows')
  Check 'ed e quella aperta' ((Win "document.querySelector('.settings-nav-on').textContent.trim()") -eq 'Impostazioni di Windows')
  Check 'c e l elenco con almeno 30 pagine' (Wait-Until { (Win "document.querySelectorAll('.winset-tile').length") -ge 30 } 6) (Win "document.querySelectorAll('.winset-tile').length")
  Check 'raggruppato (almeno 8 gruppi con il loro titolo)' ((Win "document.querySelectorAll('.winset-group').length") -ge 8)
  Check 'ogni voce ha un nome per i lettori di schermo' ((Win "Array.from(document.querySelectorAll('.winset-tile')).every(function(b){return b.getAttribute('aria-label').length>5})") -eq $true)
  Check 'non c e il vecchio pulsante «Mostra il rapporto» in questa sezione' ((Win "document.querySelectorAll('.settings-button').length") -eq 0)

  Write-Host 'B. ricerca'
  Type-Search 'luminosita'
  Check 'cercando luminosita (senza accento) resta Schermo' ((Win "Array.from(document.querySelectorAll('.winset-label')).map(function(e){return e.textContent}).join('|')") -eq 'Schermo')
  Type-Search 'zzzzz'
  Check 'una ricerca senza risultati lo dice' ((Win "document.querySelector('.settings-empty').textContent") -like 'Nessuna pagina*')
  Type-Search ''

  Write-Host 'C. apertura'
  Click-Tile 'Audio'
  Check 'un clic su Audio apre ms-settings:sound' (Wait-Until { (Last-Open) -eq 'ms-settings:sound' } 4)
  Click-Tile 'Windows Update'
  Check 'Windows Update apre ms-settings:windowsupdate' (Wait-Until { (Last-Open) -eq 'ms-settings:windowsupdate' } 4)
  Check 'un indirizzo al posto dell id e rifiutato' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('winsettings_open',{id:'ms-settings:display'})") -eq $false)
  Check 'la pagina non riceve mai gli indirizzi' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('winsettings_list').then(function(l){return JSON.stringify(l).indexOf('ms-settings')})") -eq -1)

  Write-Host 'D. la ricerca generale delle Impostazioni trova la sezione'
  Check 'cercando bluetooth compare la voce Impostazioni di Windows' (((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_search',{query:'bluetooth'})") -contains 'windows.home'))

  Write-Host 'E. altre finestre (controllo acceso)'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Check 'dalla barra winsettings_open e rifiutato' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('winsettings_open',{id:'display'}).then(function(r){return r},function(e){return 'ERR'})") -eq $false)
  Check 'dal Launcher winsettings_list non risponde' (@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('winsettings_list')" | Where-Object { $_ }).Count -eq 0)
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_NO_OPEN, env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
