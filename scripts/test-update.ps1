# Aggiornamenti: il cuore della sicurezza e dell'interfaccia, con un server LOCALE (127.0.0.1) al posto del sito vero.
# Prova: stato iniziale; «aggiornato» (stessa versione); «disponibile» (versione piu alta, con note); errori del server
# (404, JSON rotto, server spento); l'installazione di un file con firma SBAGLIATA viene RIFIUTATA e EosOS resta in vita e
# intatto; la ricerca all'avvio e spenta di default (nessuna richiesta di rete) e, accesa, fa UNA richiesta e non installa;
# i comandi si accettano solo dalla finestra Impostazioni; il blocco «Aggiornamenti» nelle Impostazioni.
# Non installa niente di vero (l'installazione con firma buona si prova nella Sandbox: scripts\prepare-sandbox-update.ps1).
# mission.json viene salvato e ripristinato.
#
# Uso: powershell -File scripts\test-update.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$serverPy = Join-Path $PSScriptRoot 'update-feed-server.py'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Check-Now { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_check').then(function(r){return r},function(e){return {state:'ERR',text:String(e)}})" }
function Status-Now { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_status')" }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$root = Join-Path $env:TEMP ("eosos-feed-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory -Force $root | Out-Null
$log = Join-Path $root 'richieste.log'
$port = 18000 + (Get-Random -Maximum 1000)
$feed = "http://127.0.0.1:$port/latest.json"
function Write-Feed([string]$version, [string]$notes = 'Novita di prova') {
  $f = @{ version = $version; notes = $notes; pub_date = '2026-11-08T12:00:00Z'; platforms = @{ 'windows-x86_64' = @{ signature = 'dW50cnVzdGVkIGNvbW1lbnQ6IGZpcm1hIGZhbHNh'; url = "http://127.0.0.1:$port/falso-setup.exe" } } }
  $f['platforms']['windows-x86_64-nsis'] = $f['platforms']['windows-x86_64']
  # Senza BOM: il feed vero non ce l'ha e serde_json lo rifiuterebbe.
  [System.IO.File]::WriteAllText((Join-Path $root 'latest.json'), ($f | ConvertTo-Json -Depth 5), (New-Object System.Text.UTF8Encoding($false)))
}
[System.IO.File]::WriteAllBytes((Join-Path $root 'falso-setup.exe'), [byte[]](0x4D, 0x5A, 0x00, 0x01, 0x02, 0x03))
$srv = $null
function Start-Eos([hashtable]$extra = @{}) {
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_UPDATE_URL = $feed
  foreach ($k in $extra.Keys) { Set-Item "env:$k" $extra[$k] }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
}
try {
  Remove-Item $mc -ErrorAction SilentlyContinue
  $srv = Start-Process python -ArgumentList '-I', "`"$serverPy`"", "`"$root`"", $port, "`"$log`"" -PassThru -WindowStyle Hidden
  Start-Sleep 2
  Start-Eos

  Write-Host 'A. stato iniziale: niente da solo'
  $s = Status-Now
  Check 'stato iniziale: mai cercato' ($s.state -eq 'idle') $s.text
  Check 'la versione in uso e quella del programma' ($s.current -match '^\d+\.\d+\.\d+')
  Start-Sleep -Milliseconds 500
  Check 'nessuna richiesta di rete all avvio (ricerca automatica spenta)' (-not (Test-Path $log))
  Check 'la ricerca automatica e spenta di default' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='general.update_auto'})[0].value})") -eq $false)

  Write-Host 'B. aggiornato'
  Write-Feed $s.current
  $r = Check-Now
  Check 'stessa versione: aggiornato' (($r.state -eq 'none') -and ($r.text -like '*aggiornato*')) $r.text
  Write-Feed '0.0.1'
  Check 'versione piu vecchia: aggiornato' ((Check-Now).state -eq 'none')

  Write-Host 'C. disponibile'
  Write-Feed '9.9.9' "Prima riga`nSeconda riga"
  $r = Check-Now
  Check 'versione piu alta: disponibile' (($r.state -eq 'available') -and ($r.version -eq '9.9.9')) $r.text
  Check 'con le note' ($r.notes -like 'Prima riga*Seconda riga')
  Check 'il testo dice quale versione' ($r.text -like '*9.9.9*')
  Check 'lo stato si ricorda' ((Status-Now).state -eq 'available')

  Write-Host 'D. firma sbagliata: l installazione viene rifiutata'
  $before = (Get-Item $exe).LastWriteTime
  $pidBefore = (Get-Process eosos | Select-Object -First 1).Id
  $err = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_install').then(function(){return 'OK-INSTALLATO'},function(e){return 'ERR:'+e})"
  Check 'update_install fallisce' ($err -like 'ERR:*') $err
  Check 'dice che l aggiornamento non e riuscito' ($err -like '*non riuscito*')
  Start-Sleep 2
  Check 'EosOS e ancora in vita (stesso processo)' (@(Get-Process eosos -ErrorAction SilentlyContinue | Where-Object { $_.Id -eq $pidBefore }).Count -eq 1)
  Check 'il programma non e stato toccato' ((Get-Item $exe).LastWriteTime -eq $before)
  Check 'lo stato e errore' ((Status-Now).state -eq 'error')
  $err2 = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_install').then(function(){return 'OK-INSTALLATO'},function(e){return 'ERR:'+e})"
  Check 'senza una ricerca nuova non si puo installare' ($err2 -like 'ERR:Prima cerca*') $err2

  Write-Host 'D2. firma ben formata ma di una ALTRA chiave: rifiutata'
  $altKey = Join-Path $root 'altra.key'
  Push-Location (Join-Path $PSScriptRoot '..')
  & npx.cmd tauri signer generate -w $altKey --ci -p prova 2>&1 | Out-Null
  $fake = Join-Path $root 'falso-setup.exe'
  & npx.cmd tauri signer sign -f $altKey -p prova $fake 2>&1 | Out-Null
  Pop-Location
  $altSig = if (Test-Path "$fake.sig") { (Get-Content "$fake.sig" -Raw).Trim() } else { '' }
  Check 'firma di prova creata con una chiave diversa' ($altSig.Length -gt 40)
  if ($altSig) {
    $f = @{ version = '9.9.9'; notes = 'x'; pub_date = '2026-11-08T12:00:00Z'; platforms = @{ 'windows-x86_64' = @{ signature = $altSig; url = "http://127.0.0.1:$port/falso-setup.exe" } } }
    $f['platforms']['windows-x86_64-nsis'] = $f['platforms']['windows-x86_64']
    [System.IO.File]::WriteAllText((Join-Path $root 'latest.json'), ($f | ConvertTo-Json -Depth 5), (New-Object System.Text.UTF8Encoding($false)))
    Check 'il feed con la firma altrui risulta disponibile' ((Check-Now).state -eq 'available')
    $err3 = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_install').then(function(){return 'OK-INSTALLATO'},function(e){return 'ERR:'+e})"
    Check 'l installazione e rifiutata per la firma' (($err3 -like 'ERR:*') -and ($err3 -match 'ignature|firma|verif')) $err3
    Check 'EosOS e ancora in vita e nulla e stato lanciato' (@(Get-Process eosos -ErrorAction SilentlyContinue | Where-Object { $_.Id -eq $pidBefore }).Count -eq 1)
  }

  Write-Host 'E. errori del server'
  Remove-Item (Join-Path $root 'latest.json')
  $r = Check-Now
  Check 'feed mancante (404): errore leggibile' (($r.state -eq 'error') -and $r.text.Length -gt 10) $r.text
  Set-Content (Join-Path $root 'latest.json') '{ non e json' -Encoding utf8
  Check 'JSON rotto: errore' ((Check-Now).state -eq 'error')
  Write-Feed '9.9.9'
  Stop-Process -Id $srv.Id -Force; Start-Sleep 1
  $r = Check-Now
  Check 'server spento: errore, nessun blocco' ($r.state -eq 'error') $r.text
  Check 'EosOS risponde ancora' ($null -ne (Status-Now))
  $srv = Start-Process python -ArgumentList '-I', "`"$serverPy`"", "`"$root`"", $port, "`"$log`"" -PassThru -WindowStyle Hidden
  Start-Sleep 2

  Write-Host 'F. le Impostazioni mostrano il blocco Aggiornamenti'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'general'})")
  Start-Sleep -Milliseconds 1500
  $has = Js 'settings' "document.querySelectorAll('.update-box').length"
  Check 'il blocco c e nella sezione Generale' ($has -eq 1)
  Check 'c e il pulsante Controlla ora' ((Js 'settings' "Array.from(document.querySelectorAll('.update-box button')).map(function(b){return b.textContent.trim()}).join('|')") -like '*Controlla ora*')
  [void](Js 'settings' "Array.from(document.querySelectorAll('.update-box button')).filter(function(b){return b.textContent.indexOf('Controlla')>=0})[0].click()")
  Check 'premendolo compare la versione disponibile' (Wait-Until { (Js 'settings' "document.querySelector('.update-text').textContent") -like '*9.9.9*' } 8)
  Check 'e il pulsante Installa' (Wait-Until { (Js 'settings' "Array.from(document.querySelectorAll('.update-box button')).map(function(b){return b.textContent.trim()}).join('|')") -like '*Installa la versione 9.9.9*' } 4)
  Check 'il testo e annunciato (role status)' ((Js 'settings' "document.querySelector('.update-text').getAttribute('role')") -eq 'status')
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_close')")

  Write-Host 'G. ricerca all avvio accesa: una richiesta, nessuna installazione'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'general.update_auto',value:true})")
  Remove-Item $log -ErrorAction SilentlyContinue
  Start-Eos
  Check 'dopo l avvio non c e ancora stata nessuna richiesta' (-not (Test-Path $log))
  Check 'entro una trentina di secondi arriva UNA richiesta del feed' (Wait-Until { (Test-Path $log) -and ((Get-Content $log) -match 'GET /latest.json').Count -ge 1 } 35)
  Start-Sleep 3
  $reqs = @(Get-Content $log)
  Check 'una sola richiesta del feed e nessun download del programma' ((($reqs -match 'latest.json').Count -eq 1) -and (($reqs -match 'falso-setup').Count -eq 0)) ($reqs -join ' ; ')
  Check 'il risultato e visibile come disponibile' ((Status-Now).state -eq 'available')

  Write-Host 'H. comandi: solo dalle Impostazioni (controllo acceso)'
  Start-Eos @{ EOSOS_TEST_STRICT = '1' }
  Check 'dalla barra update_check non fa niente' ($null -eq (Js 'bar' "window.__TAURI_INTERNALS__.invoke('update_check')"))
  Check 'dal Launcher update_status non risponde' ($null -eq (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('update_status')"))
  $e = Js 'panel' "window.__TAURI_INTERNALS__.invoke('update_install').then(function(){return 'OK'},function(x){return 'ERR:'+x})"
  Check 'dal pannello update_install e rifiutato' ($e -like 'ERR:*permess*') $e
}
finally {
  Cleanup-Eos
  if ($srv) { Stop-Process -Id $srv.Id -Force -ErrorAction SilentlyContinue }
  Remove-Item env:EOSOS_TEST_UPDATE_URL, env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
  if ($root -like "$env:TEMP\eosos-feed-*") { Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
