# Widget «Oggi» nel pannello sotto l'orologio: la data, i promemoria (aggiungi, ordine, fatto, in ritardo, togli, svuota,
# limiti, si conservano alla riapertura) e il meteo (spento di default: ZERO richieste; acceso senza citta: avviso; con la
# citta: dati da un server LOCALE che imita Open-Meteo; copia in memoria per 30 minuti; citta inesistente; server spento;
# citta non valida rifiutata). I comandi si accettano solo dal pannello. mission.json viene salvato e ripristinato.
# Nessuna notifica viene letta e nessuna richiesta esce dal PC.
#
# Uso: powershell -File scripts\test-oggi.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$serverPy = Join-Path $PSScriptRoot 'weather-mock-server.py'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class OG {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Pn([string]$expr) { Js 'panel' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Panel-Open { [OG]::Find('EosOS Pannello') -ne 0 }
function Open-Panel { Js 'bar' "document.querySelector('.clock').click()" | Out-Null; Wait-Until { (Panel-Open) -and ((Pn "document.querySelectorAll('.oggi').length") -eq 1) } 6 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }
function Reopen { Close-Panel; [void](Open-Panel); Start-Sleep -Milliseconds 700 }
function Items { (Pn "Array.from(document.querySelectorAll('.oggi-item .oggi-text')).map(function(e){return e.textContent}).join('|')") }
function Add-Rem([string]$text, [string]$time = '') {
  # React: si imposta il valore con il setter nativo e si invia l'evento, poi si invia il modulo.
  Pn "(function(){var set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;var t=document.querySelector('.oggi-input');set.call(t,'$text');t.dispatchEvent(new Event('input',{bubbles:true}));var h=document.querySelector('.oggi-hour');set.call(h,'$time');h.dispatchEvent(new Event('input',{bubbles:true}));return 1})()" | Out-Null
  Start-Sleep -Milliseconds 200
  Pn "document.querySelector('.oggi-add').click()" | Out-Null
  Start-Sleep -Milliseconds 400
}
function Setting([string]$k) { (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='$k'})[0].value})") }
function Set-S([string]$k, [string]$v) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$k',value:$v})" }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$root = Join-Path $env:TEMP ("eosos-oggi-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Force $root | Out-Null
$log = Join-Path $root 'richieste.log'
$port = 19000 + (Get-Random -Maximum 900)
$srv = $null
function Start-Eos([hashtable]$extra = @{}) {
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_WEATHER_BASE = "http://127.0.0.1:$port"
  foreach ($k in $extra.Keys) { Set-Item "env:$k" $extra[$k] }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
}
try {
  Remove-Item $mc -ErrorAction SilentlyContinue
  $srv = Start-Process python -ArgumentList '-I', "`"$serverPy`"", $port, "`"$log`"" -PassThru -WindowStyle Hidden
  Start-Sleep 2
  Start-Eos

  Write-Host 'A. il blocco Oggi, meteo spento'
  Check 'il pannello si apre con il blocco Oggi' (Open-Panel)
  Start-Sleep -Milliseconds 800
  $d = Pn "document.querySelector('.oggi-date').textContent"
  Check 'la data per esteso (giorno, numero, mese, anno)' ($d -match '^\S+ \d{1,2} \S+ \d{4}$') $d
  Check 'il meteo e spento: niente meteo e niente avviso' (((Pn "document.querySelectorAll('.oggi-weather').length") -eq 0) -and ((Pn "document.querySelectorAll('.oggi-notice').length") -eq 0))
  Check 'nessuna richiesta di rete' (-not (Test-Path $log))
  Check 'il meteo e spento di default nelle impostazioni' ((Setting 'oggi.weather') -eq $false)
  Check 'nessun promemoria all inizio' ((Pn "document.querySelector('.oggi-empty').textContent") -like 'Nessun promemoria*')
  Check 'Aggiungi e disattivato a campo vuoto' ((Pn "document.querySelector('.oggi-add').disabled") -eq $true)

  Write-Host 'B. promemoria'
  Add-Rem 'Comprare il pane'
  Add-Rem 'Medicina' '08:00'
  Add-Rem 'Vecchio' '00:01'
  Check 'con ora prima (dalla piu vicina), poi quelli senza ora' ((Items) -eq 'Vecchio|Medicina|Comprare il pane') (Items)
  Check 'l intestazione conta i da fare' ((Pn "document.querySelector('.oggi-title').textContent") -eq 'Promemoria (3)')
  Check 'campi svuotati dopo l aggiunta' ((Pn "document.querySelector('.oggi-input').value") -eq '')
  $now = Get-Date -Format 'HH:mm'
  if ($now -gt '00:01') { Check 'quello delle 00:01 e segnato in ritardo' ((Pn "document.querySelectorAll('.oggi-overdue').length") -ge 1) } else { Write-Host '  (sono le 00:00-00:01: il controllo del ritardo si salta)' }
  Pn "Array.from(document.querySelectorAll('.oggi-item')).filter(function(i){return i.textContent.indexOf('Medicina')>=0})[0].querySelector('.oggi-check').click()" | Out-Null
  Start-Sleep -Milliseconds 400
  Check 'segnato fatto: va in fondo, barrato' (((Items) -eq 'Vecchio|Comprare il pane|Medicina') -and ((Pn "document.querySelector('.oggi-item:last-child').className") -like '*oggi-done*')) (Items)
  Check 'il segno di spunta ha lo stato per i lettori di schermo' ((Pn "document.querySelector('.oggi-item:last-child .oggi-check').getAttribute('aria-checked')") -eq 'true')
  Check 'compare Svuota completati' ((Pn "document.querySelectorAll('.oggi-clear').length") -eq 1)
  Reopen
  Check 'alla riapertura i promemoria ci sono ancora' ((Items) -eq 'Vecchio|Comprare il pane|Medicina') (Items)
  Check 'sono nel file locale (e solo li)' ((Get-Content $mc -Raw) -match 'Comprare il pane')
  Pn "document.querySelector('.oggi-clear').click()" | Out-Null
  Start-Sleep -Milliseconds 400
  Check 'Svuota completati toglie solo i fatti' ((Items) -eq 'Vecchio|Comprare il pane')
  Pn "document.querySelector('.oggi-x').click()" | Out-Null
  Start-Sleep -Milliseconds 400
  Check 'la x toglie quello cliccato' ((Items) -eq 'Comprare il pane') (Items)
  $e = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('oggi_add',{text:'   ',time:null}).then(function(){return 'OK'},function(e){return 'ERR:'+e})"
  Check 'un testo vuoto e rifiutato da Rust' ($e -like 'ERR:*') $e
  $e = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('oggi_add',{text:'x',time:'25:99'}).then(function(){return 'OK'},function(e){return 'ERR:'+e})"
  Check 'un orario impossibile e rifiutato da Rust' ($e -like 'ERR:*ora*') $e
  [void](Js 'launcher' "(async function(){for(var i=0;i<60;i++){try{await window.__TAURI_INTERNALS__.invoke('oggi_add',{text:'r'+i,time:null})}catch(e){}}return 1})()")
  Check 'al massimo 50 promemoria' ((Pn "(async function(){var l=await window.__TAURI_INTERNALS__.invoke('oggi_reminders');return l.length})()") -eq 50)
  # Si ripulisce la lista per le prove seguenti.
  [void](Js 'launcher' "(async function(){var l=await window.__TAURI_INTERNALS__.invoke('oggi_reminders');for(var i=0;i<l.length;i++){await window.__TAURI_INTERNALS__.invoke('oggi_remove',{id:l[i].id})}return 1})()")

  Write-Host 'C. meteo'
  $r = Set-S 'oggi.weather' 'true'
  Check 'si accende dalle impostazioni' ($r.ok -eq $true)
  Reopen
  Check 'senza citta: avviso e nessuna richiesta' (((Pn "document.querySelector('.oggi-notice').textContent") -like '*scegli la citt*') -and (-not (Test-Path $log)))
  $r = Set-S 'oggi.city' "'<b>x</b>'"
  Check 'una citta non valida e rifiutata' ($r.ok -eq $false) $r.error
  $r = Set-S 'oggi.city' "'Roma'"
  Check 'la citta si salva' ($r.ok -eq $true)
  Reopen
  # Il confronto si fa nella pagina (PowerShell 5 rovina i caratteri non ASCII che arrivano da node).
  # Il confronto si fa nella pagina, con String.fromCharCode per il simbolo dei gradi (PowerShell 5 rovina i caratteri non ASCII).
  Check 'il meteo compare: 10 gradi, Nuvoloso, minima e massima' (Wait-Until { (Pn "(function(){var e=document.querySelector('.oggi-weather');if(!e)return false;var d=String.fromCharCode(176);var t=e.textContent;return t.indexOf('10'+d+'Roma')>=0&&t.indexOf('Nuvoloso')>=0&&t.indexOf('10'+d+' / 17'+d)>=0})()") -eq $true } 8)
  Check 'nome accessibile completo' ((Pn "(function(){var d=String.fromCharCode(176);return document.querySelector('.oggi-weather').getAttribute('aria-label')==='Roma: 10'+d+' Nuvoloso, minima 10'+d+', massima 17'+d})()") -eq $true)
  $reqs = @(Get-Content $log)
  Check 'una ricerca della citta e una delle previsioni' ((($reqs -match '/v1/search Roma').Count -eq 1) -and (($reqs -match '/v1/forecast').Count -eq 1)) ($reqs -join ' ; ')
  Reopen
  Start-Sleep -Milliseconds 800
  Check 'riaprendo entro 30 minuti non si richiede niente (copia in memoria)' (@(Get-Content $log).Count -eq 2)
  Check 'il meteo c e ancora' ((Pn "document.querySelectorAll('.oggi-weather').length") -eq 1)
  [void](Set-S 'oggi.city' "'Nowhereville'")
  Reopen
  Check 'citta inesistente: messaggio chiaro' (Wait-Until { (Pn "(document.querySelector('.oggi-notice')||{textContent:''}).textContent") -like '*Non trovo la citt*Nowhereville*' } 8)
  Stop-Process -Id $srv.Id -Force; Start-Sleep 1
  [void](Set-S 'oggi.city' "'Milano'")
  Reopen
  Check 'server spento: errore leggibile, il pannello funziona ancora' (Wait-Until { (Pn "(document.querySelector('.oggi-notice')||{textContent:''}).textContent") -like 'Meteo non disponibile*' } 15)
  Check 'i promemoria funzionano comunque' ((Pn "document.querySelectorAll('.oggi-form').length") -eq 1)
  [void](Set-S 'oggi.weather' 'false')
  Reopen
  Check 'spento di nuovo: niente meteo' (((Pn "document.querySelectorAll('.oggi-weather').length") -eq 0) -and ((Pn "document.querySelectorAll('.oggi-notice').length") -eq 0))
  Close-Panel

  Write-Host 'D. comandi: solo dal pannello (controllo acceso)'
  Start-Eos @{ EOSOS_TEST_STRICT = '1' }
  $e = Js 'bar' "window.__TAURI_INTERNALS__.invoke('oggi_add',{text:'intruso',time:null}).then(function(){return 'OK'},function(e){return 'ERR:'+e})"
  Check 'dalla barra oggi_add e rifiutato' ($e -like 'ERR:*permess*') $e
  Check 'dal Launcher oggi_reminders non risponde' (@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('oggi_reminders')").Count -le 1)
  $e = Js 'panel' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'oggi.city',value:'Roma'}).then(function(r){return r.ok?'OK':'NO:'+r.error},function(e){return 'ERR:'+e})"
  Check 'dal pannello la citta non si cambia (solo le Impostazioni)' ($e -like 'NO:*permess*') $e
}
finally {
  Cleanup-Eos
  if ($srv) { Stop-Process -Id $srv.Id -Force -ErrorAction SilentlyContinue }
  Remove-Item env:EOSOS_TEST_WEATHER_BASE, env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
  if ($root -like "$env:TEMP\eosos-oggi-*") { Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
