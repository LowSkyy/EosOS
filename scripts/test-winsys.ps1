# Impostazioni di Windows dentro EosOS (Impostazioni → Windows): informazioni sul sistema, aspetto, alimentazione, app all'avvio, archiviazione, schermi.
# La prova NON cambia nulla di visibile: le scritture si fanno solo riscrivendo il valore che c'e' gia (tema, tempi di alimentazione) e per le app all'avvio
# solo su una voce di prova MIA (HKCU\...\Run, valore «EosOS-Prova-Winsys», che si toglie alla fine). Si controlla anche che nomi o valori inventati
# non scrivano niente e che le altre finestre non possano chiamare i comandi.
#
# Uso: powershell -File scripts\test-winsys.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Windows.Forms
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }; $false }
function Inv([string]$cmd, [string]$extra = '') { Js 'settings' "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
$runKey = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Run'
$testName = 'EosOS-Prova-Winsys'
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  Remove-ItemProperty -Path $runKey -Name $testName -ErrorAction SilentlyContinue
  New-ItemProperty -Path $runKey -Name $testName -Value 'C:\Windows\System32\notepad.exe' -PropertyType String -Force | Out-Null
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 13

  Write-Host 'A. letture'
  $i = Inv 'winsys_info'
  Check 'informazioni: nome del dispositivo, utente e Windows' ((-not [string]::IsNullOrEmpty($i.device)) -and ($i.user -eq $env:USERNAME) -and ($i.os -like 'Windows*')) ("$($i.device) / $($i.os)")
  Check 'processore e memoria' ((-not [string]::IsNullOrEmpty($i.cpu)) -and ($i.memory -match '(GB|MB|TB)')) ("$($i.cpu) / $($i.memory)")
  $d = @(Inv 'winsys_drives')
  Check 'almeno un disco con spazio totale e libero' (($d.Count -ge 1) -and ($d[0].total -match 'GB|TB|MB') -and ($d[0].used_percent -ge 0) -and ($d[0].used_percent -le 100)) ("dischi: " + $d.Count)
  $dp = @(Inv 'winsys_displays')
  Check 'gli schermi sono quelli del sistema' ($dp.Count -eq [System.Windows.Forms.Screen]::AllScreens.Count -or $dp.Count -ge 1) ("schermi: " + $dp.Count)
  Check 'uno e il principale' (@($dp | Where-Object { $_.primary -eq $true }).Count -ge 1)

  Write-Host 'B. tema (si riscrive il valore che c e gia: niente cambia)'
  $t = Inv 'winsys_theme'
  Check 'il tema si legge' ($null -ne $t.apps_light)
  foreach ($w in @(@('apps', $t.apps_light), @('system', $t.system_light), @('transparency', $t.transparency))) {
    $ok = Inv 'winsys_theme_set' (",{which:'" + $w[0] + "',on:" + ([string]$w[1]).ToLower() + "}")
    Check ("riscrivere " + $w[0] + " com e: accettato") ($ok -eq $true)
  }
  $t2 = Inv 'winsys_theme'
  Check 'dopo le riscritture il tema e identico' (($t2.apps_light -eq $t.apps_light) -and ($t2.system_light -eq $t.system_light) -and ($t2.transparency -eq $t.transparency))
  Check 'un id inventato non scrive' ((Inv 'winsys_theme_set' ",{which:'boh',on:true}") -eq $false)

  Write-Host 'C. alimentazione (idem: si riscrive il valore che c e)'
  $p = Inv 'winsys_power'
  Check 'i tempi si leggono' (($null -ne $p.screen_ac) -and ($null -ne $p.sleep_ac))
  $ok = Inv 'winsys_power_set' (",{which:'screen-ac',minutes:" + $p.screen_ac + "}")
  Check 'riscrivere lo spegnimento dello schermo com e: accettato' ($ok -eq $true)
  $p2 = Inv 'winsys_power'
  Check 'e il valore e lo stesso' ($p2.screen_ac -eq $p.screen_ac)
  Check 'minuti non ammessi (7) rifiutati' ((Inv 'winsys_power_set' ",{which:'screen-ac',minutes:7}") -eq $false)
  Check 'un id inventato rifiutato' ((Inv 'winsys_power_set' ",{which:'boh',minutes:5}") -eq $false)

  Write-Host 'D. app all avvio (solo la mia voce di prova)'
  $su = @(Inv 'winsys_startup')
  $mine = $su | Where-Object { $_.name -eq $testName }
  Check 'la voce di prova e nell elenco, accesa' (($null -ne $mine) -and ($mine.enabled -eq $true))
  Check 'la spegni' ((Inv 'winsys_startup_set' (",{name:'$testName',on:false}")) -eq $true)
  Check 'ora risulta spenta' (Wait-Until { ((@(Inv 'winsys_startup') | Where-Object { $_.name -eq $testName }).enabled) -eq $false } 4)
  Check 'la riaccendi' ((Inv 'winsys_startup_set' (",{name:'$testName',on:true}")) -eq $true)
  Check 'ora risulta accesa' (Wait-Until { ((@(Inv 'winsys_startup') | Where-Object { $_.name -eq $testName }).enabled) -eq $true } 4)
  Check 'un nome che non c e non scrive' ((Inv 'winsys_startup_set' ",{name:'nome-che-non-esiste',on:false}") -eq $false)

  Write-Host 'E. la pagina'
  Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'windows'})" | Out-Null
  Start-Sleep 2
  $sec = Js 'settings' "document.querySelectorAll('.winsys-card').length"
  Check 'la pagina Windows mostra le schede (informazioni, aspetto, alimentazione, avvio, archiviazione, schermi)' (Wait-Until { (Js 'settings' "document.querySelectorAll('.winsys-card').length") -ge 6 } 10) ("schede: " + (Js 'settings' "document.querySelectorAll('.winsys-card').length"))
  Check 'ci sono gli interruttori del tema' ((Js 'settings' "document.querySelectorAll('.winsys-card[aria-label=\'Aspetto di Windows\'] .switch').length") -eq 3)
  Check 'la nota sulle altre pagine c e' ((Js 'settings' "(document.querySelector('.winset-note')||{textContent:''}).textContent") -like 'Queste pagine*')

  Write-Host 'F. controllo accessi'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Check 'dalla barra winsys_theme_set non scrive' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('winsys_theme_set',{which:'apps',on:true}).then(function(r){return r},function(e){return false})") -ne $true)
  Check 'dal Launcher winsys_startup_set non scrive' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('winsys_startup_set',{name:'$testName',on:false}).then(function(r){return r},function(e){return false})") -ne $true)
  Check 'e la voce di prova e rimasta accesa' (((Get-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' -Name $testName -ErrorAction SilentlyContinue).$testName | Select -First 1) -ne 3)
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-ItemProperty -Path $runKey -Name $testName -ErrorAction SilentlyContinue
  Remove-ItemProperty -Path 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run' -Name $testName -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
