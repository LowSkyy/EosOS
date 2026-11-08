# Passata finale: i comandi sensibili si possono chiamare solo dalla finestra giusta (EOSOS_TEST_STRICT=1 accende il
# controllo anche nella build di prova), e spegnere/riavviare chiede anche un clic in una finestra di Windows.
# Si prova dalle pagine vere (barra, pannello, Launcher) con la console di sviluppo. Le azioni di alimentazione non
# vengono mai eseguite (la build di prova scrive solo una riga in %TEMP%\eosos-actions.log). mission.json e launcher.json
# vengono salvati e ripristinati; nessuna notifica viene letta (la lettura resta spenta).
#
# Uso: powershell -File scripts\test-cmd-policy.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
# Chiama un comando e restituisce la risposta, oppure «ERR:testo» se Tauri lo ha rifiutato.
function Call([string]$view, [string]$cmd, [string]$ca = '{}') { Js $view "window.__TAURI_INTERNALS__.invoke('$cmd',$ca).then(function(r){return r},function(e){return 'ERR:'+e})" }
function Setting([string]$k) { $a = Call 'panel' 'settings_all'; ($a | Where-Object { $_.key -eq $k }).value }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$ln = Join-Path $env:LOCALAPPDATA 'EosOS\launcher.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$bkLn = if (Test-Path $ln) { [System.IO.File]::ReadAllBytes($ln) } else { $null }
$log = Join-Path $env:TEMP 'eosos-actions.log'
$bkLog = if (Test-Path $log) { [System.IO.File]::ReadAllBytes($log) } else { $null }
function Start-Eos([hashtable]$extra = @{}) {
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  foreach ($k in $extra.Keys) { Set-Item "env:$k" $extra[$k] }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
}
try {
  Remove-Item $mc, $ln -ErrorAction SilentlyContinue
  Start-Eos

  Write-Host 'A. la barra (finestra di poca fiducia) non arriva ai comandi sensibili'
  foreach ($c in 'settings_all', 'settings_search', 'notif_list', 'launcher_search', 'launcher_files', 'launcher_clip_use', 'launcher_launch', 'launcher_action') {
    $a = switch ($c) { 'settings_search' { "{query:'dock'}" } 'launcher_search' { "{query:'a'}" } 'launcher_files' { "{query:'a'}" } 'launcher_clip_use' { "{id:'clip:1'}" } 'launcher_launch' { "{id:'x'}" } 'launcher_action' { "{id:'action:lock'}" } default { '{}' } }
    $r = Call 'bar' $c $a
    $blocked = ($null -eq $r) -or ($r -is [string] -and ($r -eq 'unknown' -or $r -like 'ERR:*')) -or (($r -is [array] -or $r -is [System.Collections.IEnumerable]) -and @($r).Count -eq 0) -or (($r.state -eq 'off') -and (@($r.groups).Count -eq 0)) -or ($r -eq $false)
    Check "bar: $c e bloccato" $blocked ("risposta: " + ($r | Out-String).Trim())
  }
  $r = Call 'bar' 'settings_set' "{key:'notif.read',value:true}"
  Check 'bar: settings_set notif.read rifiutato' ($r.ok -eq $false) ($r.error)
  $r = Call 'bar' 'launcher_clip_set' '{on:true}'
  Check 'bar: launcher_clip_set non accende gli appunti' ((Setting 'launcher.clipboard') -eq $false)
  Check 'la lettura delle notifiche e ancora spenta' ((Setting 'notif.read') -eq $false)

  Write-Host 'B. il pannello sotto l orologio puo solo i due interruttori delle notifiche'
  $r = Call 'panel' 'settings_set' "{key:'launcher.clipboard',value:true}"
  Check 'panel: launcher.clipboard rifiutato' ($r.ok -eq $false) ($r.error)
  $r = Call 'panel' 'settings_set' "{key:'dock.pinned',value:['x']}"
  Check 'panel: dock.pinned rifiutato' ($r.ok -eq $false)
  $r = Call 'panel' 'settings_set' "{key:'general.autostart',value:true}"
  Check 'panel: general.autostart rifiutato' ($r.ok -eq $false)
  Check 'panel: gli appunti sono rimasti spenti' ((Setting 'launcher.clipboard') -eq $false)
  $r = Call 'panel' 'settings_set' "{key:'notif.dnd',value:true}"
  Check 'panel: notif.dnd ammesso' (($r.ok -eq $true) -and ((Setting 'notif.dnd') -eq $true))
  [void](Call 'panel' 'settings_set' "{key:'notif.dnd',value:false}")
  Check 'panel: notif.dnd rimesso a spento' ((Setting 'notif.dnd') -eq $false)
  Check 'panel: non puo usare i comandi del Launcher' ((Call 'panel' 'launcher_action' "{id:'action:lock'}") -eq 'unknown')

  Write-Host 'C. il Launcher non arriva alle impostazioni ne alle notifiche'
  $r = Call 'launcher' 'settings_set' "{key:'notif.read',value:true}"
  Check 'launcher: settings_set rifiutato' ($r.ok -eq $false)
  $r = Call 'launcher' 'notif_list'
  Check 'launcher: notif_list vuoto' (($r.state -eq 'off') -and (@($r.groups).Count -eq 0))
  $all = Call 'launcher' 'settings_all'
  Check 'launcher: settings_all vuoto' (($null -eq $all) -or (@($all | Where-Object { $_ }).Count -eq 0))
  [void](Call 'launcher' 'launcher_clip_set' '{on:true}')
  Check 'launcher: il suo comando launcher_clip_set funziona' (Setting 'launcher.clipboard')
  [void](Call 'launcher' 'launcher_clip_set' '{on:false}')
  Check 'launcher: e si rimette a spento' ((Setting 'launcher.clipboard') -eq $false)

  Write-Host 'D. spegnere o riavviare: serve anche la finestra di Windows'
  Remove-Item $log -ErrorAction SilentlyContinue
  $r1 = Call 'launcher' 'launcher_action' "{id:'action:restart'}"
  Check 'prima richiesta: serve conferma' ($r1 -eq 'confirm') $r1
  $r2 = Call 'launcher' 'launcher_action' "{id:'action:restart'}"
  Check 'seconda richiesta con la finestra di Windows accettata: eseguita' ($r2 -eq 'done') $r2
  Check 'la riga e nel registro di prova' ((Test-Path $log) -and ((Get-Content $log -Raw) -match 'action:restart'))
  Start-Eos @{ EOSOS_TEST_CONFIRM = 'no' }
  Remove-Item $log -ErrorAction SilentlyContinue
  [void](Call 'launcher' 'launcher_action' "{id:'action:shutdown'}")
  $r2 = Call 'launcher' 'launcher_action' "{id:'action:shutdown'}"
  Check 'con «No» nella finestra di Windows: non eseguita' ($r2 -eq 'failed') $r2
  Check 'e nulla nel registro di prova' (-not (Test-Path $log) -or -not ((Get-Content $log -Raw) -match 'shutdown'))
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT, env:EOSOS_TEST_CONFIRM -ErrorAction SilentlyContinue
  Remove-Item $mc, $ln -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
  if ($null -ne $bkLn) { [System.IO.File]::WriteAllBytes($ln, $bkLn) }
  Remove-Item $log -ErrorAction SilentlyContinue
  if ($null -ne $bkLog) { [System.IO.File]::WriteAllBytes($log, $bkLog) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
