# Badge delle notifiche sulle icone del Dock: nel Dock c'e' (fissata per la prova) l'icona di Windows PowerShell e si mandano toast VERI ma
# SOLO miei (titoli «EosOS-B-...», inviati con l'identificatore di PowerShell, rimossi alla fine). Si controlla: niente badge con la lettura delle
# notifiche spenta (privacy: non si legge nulla), il badge col numero giusto quando e' accesa, scompare con Non disturbare, con l'impostazione
# spenta e quando le notifiche vengono tolte; il testo accessibile dice quante sono; la barra non puo' chiedere i badge. dock.json e mission.json
# sono salvati e ripristinati; non si clicca su icone vere.
#
# Uso: powershell -File scripts\test-dock-badges.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 300 }; $false }
function Set-S([string]$key, [string]$jsValue) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$key',value:$jsValue})" }
# Il badge sull'icona di PowerShell nel Dock principale (testo, o '' se non c'e').
$psSel = ".dock:not(.dock-ghost) .dock-item[data-label=\'Windows PowerShell\']"
function Badge-Text { [string](Js 'dock' "(function(){var i=document.querySelector('$psSel');var b=i&&i.querySelector('.dock-badge');return b?b.textContent:''})()") }
function Badge-Aria { [string](Js 'dock' "(function(){var i=document.querySelector('$psSel');return i?i.getAttribute('aria-label'):''})()") }
function Has-Icon { (Js 'dock' "document.querySelector('$psSel')!==null") -eq $true }

$aumid = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
$toast = Join-Path $env:TEMP 'eosos-badge-toast.ps1'
Set-Content $toast @'
$Tag = $env:EOS_TAG; $Title = $env:EOS_T
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml('<toast><visual><binding template="ToastGeneric"><text>' + [System.Security.SecurityElement]::Escape($Title) + '</text></binding></visual></toast>')
$n = New-Object Windows.UI.Notifications.ToastNotification $xml
$n.Tag = $Tag; $n.Group = 'eosos-badge'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($env:EOS_AUMID).Show($n)
'@ -Encoding UTF8
$remove = Join-Path $env:TEMP 'eosos-badge-remove.ps1'
Set-Content $remove @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
foreach ($tag in 'b-a', 'b-b', 'b-c') { try { [Windows.UI.Notifications.ToastNotificationManager]::History.Remove($tag, 'eosos-badge', $env:EOS_AUMID) } catch {} }
'@ -Encoding UTF8
function Send([string]$tag, [string]$title) { $env:EOS_TAG = $tag; $env:EOS_T = $title; & powershell -NoProfile -ExecutionPolicy Bypass -File $toast }
function Remove-Mine { & powershell -NoProfile -ExecutionPolicy Bypass -File $remove }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$mc = Join-Path $dir 'mission.json'; $dj = Join-Path $dir 'dock.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$psPath = 'C:\Windows\System32\WindowsPowerShell\v1.0\powershell.exe'
$psJson = $psPath.Replace('\', '\\')   # con le barre raddoppiate: vale sia per il JSON sia per un testo JS tra apici
try {
  $env:EOS_AUMID = $aumid; $env:EOSOS_TEST_NO_OPEN = '1'
  Remove-Mine
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  # Il Dock di prova: solo PowerShell fissato (cosi' c'e' un'icona che le notifiche mie riconoscono).
  [System.IO.File]::WriteAllText($dj, ('{ "version": 1, "pinned": [ { "name": "Windows PowerShell", "path": "' + $psJson + '" } ] }'), (New-Object System.Text.UTF8Encoding $false))
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 13

  Write-Host 'A. lettura spenta: nessun badge, niente letto'
  Check 'l icona di PowerShell e nel Dock' (Wait-Until { Has-Icon } 8)
  Send 'b-a' 'EosOS-B-uno'; Send 'b-b' 'EosOS-B-due'
  Start-Sleep 6
  Check 'con «Mostra le notifiche» spento il Dock non mostra badge' ((Badge-Text) -eq '')
  Check 'e il comando lo conferma (lista vuota)' ((Js 'dock' "window.__TAURI_INTERNALS__.invoke('dock_badges',{items:[{id:'x',name:'Windows PowerShell',path:'$psJson'}]}).then(function(r){return r.length})") -eq 0)

  Write-Host 'B. lettura accesa: il badge col numero'
  [void](Set-S 'notif.read' 'true')
  Check 'il badge compare e dice 2' (Wait-Until { (Badge-Text) -eq '2' } 14) ("testo: '" + (Badge-Text) + "'")
  Check 'il testo accessibile dice quante notifiche' ((Badge-Aria) -eq 'Windows PowerShell, 2 notifiche') (Badge-Aria)
  Send 'b-c' 'EosOS-B-tre'
  Check 'una terza notifica: il badge sale a 3' (Wait-Until { (Badge-Text) -eq '3' } 14) ("testo: '" + (Badge-Text) + "'")
  Check 'un solo badge sull icona' ((Js 'dock' "document.querySelectorAll('$psSel .dock-badge').length") -eq 1)

  Write-Host 'C. Non disturbare e impostazione'
  [void](Set-S 'notif.dnd' 'true')
  Check 'con Non disturbare il badge sparisce' (Wait-Until { (Badge-Text) -eq '' } 10)
  [void](Set-S 'notif.dnd' 'false')
  Check 'spento Non disturbare torna' (Wait-Until { (Badge-Text) -eq '3' } 14)
  $r = Set-S 'dock.badges' 'false'
  Check 'l impostazione «Badge delle notifiche» si spegne' ($r.ok -eq $true)
  Check 'e il badge sparisce' (Wait-Until { (Badge-Text) -eq '' } 10)
  [void](Set-S 'dock.badges' 'true')
  Check 'riaccesa, torna' (Wait-Until { (Badge-Text) -eq '3' } 14)

  Write-Host 'D. notifiche tolte'
  Remove-Mine
  Check 'tolte le notifiche mie il badge sparisce' (Wait-Until { (Badge-Text) -eq '' } 14) ("testo: '" + (Badge-Text) + "'")

  Write-Host 'E. controllo accessi'
  Cleanup-Eos; Start-Sleep 1
  Send 'b-a' 'EosOS-B-uno'
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $ask = "window.__TAURI_INTERNALS__.invoke('dock_badges',{items:[{id:'x',name:'Windows PowerShell',path:'$psJson'}]}).then(function(r){return r.length},function(e){return -1})"
  Check 'dal Dock il conteggio c e (1 icona con badge)' (Wait-Until { (Js 'dock' $ask) -eq 1 } 14)
  Check 'dalla barra la risposta e vuota (non e del Dock)' ((Js 'bar' $ask) -le 0)
  Check 'dal Launcher neppure' ((Js 'launcher' $ask) -le 0)
}
finally {
  Remove-Mine
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT, env:EOS_AUMID, env:EOSOS_TEST_NO_OPEN -ErrorAction SilentlyContinue
  Remove-Item $mc, $dj, $toast, $remove -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
