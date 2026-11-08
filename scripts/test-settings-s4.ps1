# Verifica S4 delle impostazioni: sezione Generale (avvio automatico con Windows, lingua, rapporto diagnostico).
# ATTENZIONE: questa prova scrive e toglie DAVVERO il valore "EosOS" nella chiave Run dell'utente
# (HKCU\Software\Microsoft\Windows\CurrentVersion\Run) e il suo eventuale blocco in StartupApproved: lo stato di
# partenza di quei due valori viene salvato e RIPRISTINATO alla fine (compresa l'assenza). Gli altri valori della
# chiave Run non si toccano e a fine prova si controlla che siano identici. I quattro file di impostazioni di EosOS
# vengono anch'essi salvati e ripristinati. Il rapporto diagnostico si scrive ma NON si apre in nessun programma.
#
# Uso: powershell -File scripts\test-settings-s4.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Win([string]$expr) { Js 'settings' $expr }
function Inv([string]$expr) { Js 'launcher' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Val([string]$key) { (@(Inv "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq $key }).value }

$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$apprKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run'
function Run-Value { try { (Get-ItemProperty -Path $runKey -Name 'EosOS' -ErrorAction Stop).EosOS } catch { $null } }
function Appr-Value { try { (Get-ItemProperty -Path $apprKey -Name 'EosOS' -ErrorAction Stop).EosOS } catch { $null } }
function Others { if (Test-Path $runKey) { (Get-Item $runKey).GetValueNames() | Where-Object { $_ -ne 'EosOS' } | Sort-Object | ForEach-Object { $_ + '=' + (Get-ItemProperty $runKey).$_ } } else { @() } }

$bk = Join-Path $env:TEMP 'eosos-s4-backup'; New-Item -ItemType Directory -Force -Path $bk | Out-Null
& reg export 'HKCU\Software\Microsoft\Windows\CurrentVersion\Run' (Join-Path $bk 'run.reg') /y | Out-Null
if (Test-Path $apprKey) { & reg export 'HKCU\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' (Join-Path $bk 'approved.reg') /y | Out-Null }
$origRun = Run-Value; $origAppr = Appr-Value; $origOthers = @(Others)
$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  # Partenza pulita per EosOS (lo stato vero e' salvato sopra e tornera' alla fine).
  Remove-ItemProperty -Path $runKey -Name 'EosOS' -ErrorAction SilentlyContinue
  Remove-ItemProperty -Path $apprKey -Name 'EosOS' -ErrorAction SilentlyContinue
  $env:EOSOS_TEST_NO_OPEN = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_open',{section:null})"); Start-Sleep -Milliseconds 1200

  Write-Host 'A. la sezione Generale'
  Check 'e la prima sezione' ((Win "document.querySelector('.settings-h').textContent") -eq 'Generale')
  Check 'ha avvio automatico, lingua e rapporto' (((Win "Array.from(document.querySelectorAll('.settings-label')).map(function(e){return e.childNodes[0].textContent})") -join '|') -like 'Apri EosOS*|Lingua|Rapporto sui desktop virtuali')
  Check 'la lingua e solo Italiano' (((Win "Array.from(document.querySelector('select').options).map(function(o){return o.textContent})") -join '|') -eq 'Italiano')

  Write-Host 'B. avvio automatico'
  Check 'all inizio e spento' ((Val 'general.autostart') -eq $false -and $null -eq (Run-Value))
  Win "document.querySelector('[role=switch]').click()" | Out-Null
  Check 'cliccando l interruttore compare il valore EosOS nella chiave Run dell utente' (Wait-Until { $null -ne (Run-Value) } 4)
  $rv = Run-Value
  Check 'e punta a questo eseguibile, fra virgolette' ($rv -eq ('"' + $exe + '"')) $rv
  Check 'la pagina lo mostra acceso' (Wait-Until { (Win "document.querySelector('[role=switch]').getAttribute('aria-checked')") -eq 'true' } 4)
  Check 'gli altri valori della chiave Run sono intatti' ((@(Others) -join '|') -eq ($origOthers -join '|'))
  # Gestione attivita lo disattiva: per le impostazioni e spento.
  if (-not (Test-Path $apprKey)) { New-Item -Path $apprKey | Out-Null }   # MAI -Force su una chiave esistente: cancella tutti i suoi valori
  Set-ItemProperty -Path $apprKey -Name 'EosOS' -Type Binary -Value ([byte[]](3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0))
  Check 'bloccato da Gestione attivita: risulta spento' (Wait-Until { (Val 'general.autostart') -eq $false } 4)
  [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'general.autostart',value:true})")
  Check 'riaccenderlo toglie il blocco e risulta acceso' (Wait-Until { ((Val 'general.autostart') -eq $true) -and ($null -eq (Appr-Value)) } 4)
  Win "document.querySelector('[role=switch]').click()" | Out-Null
  Check 'spegnere toglie il valore' (Wait-Until { $null -eq (Run-Value) } 4)
  Check 'e gli altri valori sono ancora intatti' ((@(Others) -join '|') -eq ($origOthers -join '|'))
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'general.autostart',value:'si'})"
  Check 'un valore non booleano e rifiutato' ($r.ok -eq $false)
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'general.language',value:'en'})"
  Check 'una lingua che non c e e rifiutata' ($r.ok -eq $false)

  Write-Host 'C. rapporto sui desktop virtuali (si scrive, non si apre)'
  $rep = Join-Path $dir 'vdesk-report.json'
  $before = if (Test-Path $rep) { (Get-Item $rep).LastWriteTime } else { [datetime]::MinValue }
  Win "Array.from(document.querySelectorAll('.settings-button')).find(function(b){return b.textContent.indexOf('rapporto')>0}).click()" | Out-Null
  Check 'il file del rapporto viene scritto' (Wait-Until { (Test-Path $rep) -and ((Get-Item $rep).LastWriteTime -gt $before) } 8)
  $json = Get-Content $rep -Raw
  Check 'contiene la versione di Windows e i controlli' (($json -match 'build') -and ($json -match 'attempts'))
  Check 'non contiene il nome utente ne percorsi personali' (-not ($json -match [regex]::Escape($env:USERNAME)))
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_action',{key:'dock.autohide'})"
  Check 'un interruttore non si puo premere come pulsante' ($r.ok -eq $false)
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'general.report',value:true})"
  Check 'e un pulsante non si imposta come un valore' ($r.ok -eq $false)

  Write-Host 'D. ricerca'
  $res = @(Inv "window.__TAURI_INTERNALS__.invoke('settings_search',{query:'avvio automatico'})" | ForEach-Object { $_ })
  Check 'cercando avvio automatico la prima e l avvio all accesso' ($res[0] -eq 'general.autostart') ($res -join ',')
  $res = @(Inv "window.__TAURI_INTERNALS__.invoke('settings_search',{query:'diagnostica'})" | ForEach-Object { $_ })
  Check 'cercando diagnostica si trova il rapporto' ($res -contains 'general.report')
}
finally {
  Remove-Item env:EOSOS_TEST_NO_OPEN -ErrorAction SilentlyContinue
  Cleanup-Eos
  # Ripristino dei due valori di EosOS nel registro (compresa l'assenza).
  Remove-ItemProperty -Path $runKey -Name 'EosOS' -ErrorAction SilentlyContinue
  Remove-ItemProperty -Path $apprKey -Name 'EosOS' -ErrorAction SilentlyContinue
  if ($null -ne $origRun) { Set-ItemProperty -Path $runKey -Name 'EosOS' -Value $origRun }
  if ($null -ne $origAppr) { if (-not (Test-Path $apprKey)) { New-Item -Path $apprKey | Out-Null }; Set-ItemProperty -Path $apprKey -Name 'EosOS' -Type Binary -Value ([byte[]]$origAppr) }
  foreach ($f in $files) {
    $p = Join-Path $dir $f
    Remove-Item $p -ErrorAction SilentlyContinue
    if ($null -ne $backup[$f]) { [System.IO.File]::WriteAllBytes($p, $backup[$f]) }
  }
}
$regSame = ((Run-Value) -eq $origRun) -and ((Appr-Value) -eq $origAppr -or ($null -ne $origAppr -and $null -ne (Appr-Value) -and (@(Appr-Value) -join ',') -eq (@($origAppr) -join ','))) -and ((@(Others) -join '|') -eq ($origOthers -join '|'))
# Rete di sicurezza: se qualcosa non torna, si reimporta la copia COMPLETA fatta all'inizio.
if (-not $regSame) {
  Write-Host 'Il registro non e identico: reimporto la copia di sicurezza completa' -ForegroundColor Yellow
  foreach ($f in 'run.reg', 'approved.reg') { $fp = Join-Path $bk $f; if (Test-Path $fp) { & reg import $fp | Out-Null } }
  Remove-ItemProperty -Path $runKey -Name 'EosOS' -ErrorAction SilentlyContinue
  if ($null -ne $origRun) { Set-ItemProperty -Path $runKey -Name 'EosOS' -Value $origRun }
}
Write-Host ("Registro ripristinato identico: " + $regSame)
$same = $true
foreach ($f in $files) { $p = Join-Path $dir $f; $now = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null }; if (($null -eq $now) -ne ($null -eq $backup[$f])) { $same = $false } elseif ($null -ne $now -and -not [System.Linq.Enumerable]::SequenceEqual([byte[]]$now, [byte[]]$backup[$f])) { $same = $false } }
Write-Host ("File dell utente ripristinati identici: " + $same)
if ($script:fail -eq 0 -and $same -and $regSame) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
