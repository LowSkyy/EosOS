# Centro notifiche, fase N4: l'elenco delle notifiche nel pannello sotto l'orologio, sopra il calendario. Con notifiche
# VERE ma SOLO mie (tre toast mandati da una finestra di PowerShell con titoli «EosOS-N4-...»): si leggono dalla pagina del
# pannello guardando SOLO il gruppo dell'app PowerShell, le notifiche di altre app non si leggono qui ne si stampano.
# Il «×» di gruppo si prova solo se nel gruppo ci sono soltanto le mie. «Apri l'app» non apre niente di vero:
# EOSOS_TEST_NO_OPEN fa ricordare l'identificatore invece di lanciare il programma. mission.json viene ripristinato.
# Durante la prova compaiono per un attimo gli avvisi a schermo.
#
# Uso: powershell -File scripts\test-notif-panel.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class NP {
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
function Panel-Open { [NP]::Find('EosOS Pannello') -ne 0 }
function Open-Panel { Js 'bar' "document.querySelector('.clock').click()" | Out-Null; Wait-Until { (Panel-Open) -and ((Pn "document.querySelectorAll('.notifs').length") -eq 1) } 6 }
# Il gruppo dell'app PowerShell nel pannello: i titoli delle sue notifiche (solo quelle del gruppo, mai altri gruppi).
$myGroup = "Array.from(document.querySelectorAll('.notifs-group')).filter(function(g){return g.getAttribute('aria-label')==='Windows PowerShell'})[0]"
function My-Titles { @(Pn "(function(){var g=$myGroup;return g?Array.from(g.querySelectorAll('.notifs-ttl')).map(function(e){return e.textContent}):[]})()" | ForEach-Object { $_ }) }

$aumid = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
$toast = Join-Path $env:TEMP 'eosos-n4-toast.ps1'
Set-Content $toast @'
$Tag = $env:EOS_TAG; $Title = $env:EOS_T; $Line1 = $env:EOS_L1
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$t = '<text>' + [System.Security.SecurityElement]::Escape($Title) + '</text>'
if ($Line1) { $t += '<text>' + [System.Security.SecurityElement]::Escape($Line1) + '</text>' }
$xml.LoadXml('<toast><visual><binding template="ToastGeneric">' + $t + '</binding></visual></toast>')
$n = New-Object Windows.UI.Notifications.ToastNotification $xml
$n.Tag = $Tag; $n.Group = 'eosos-n4'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($env:EOS_AUMID).Show($n)
'@ -Encoding UTF8
$remove = Join-Path $env:TEMP 'eosos-n4-remove.ps1'
Set-Content $remove @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
foreach ($tag in 'n4-a', 'n4-b', 'n4-c', 'n4-d', 'n4-e') { try { [Windows.UI.Notifications.ToastNotificationManager]::History.Remove($tag, 'eosos-n4', $env:EOS_AUMID) } catch {} }
'@ -Encoding UTF8
function Send([string]$tag, [string]$title, [string]$l1 = '') { $env:EOS_TAG = $tag; $env:EOS_T = $title; $env:EOS_L1 = $l1; & powershell -NoProfile -ExecutionPolicy Bypass -File $toast }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOS_AUMID = $aumid; $env:EOSOS_TEST_NO_OPEN = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. lettura spenta (predefinita): il pannello lo dice e offre di accenderla'
  Check 'il pannello si apre con le notifiche sopra il calendario' (Open-Panel)
  Check 'intestazione Notifiche' ((Pn "document.querySelector('.notifs-title').textContent") -eq 'Notifiche')
  Check 'dice che la lettura e spenta' ((Pn "document.querySelector('.notifs-off p').textContent") -like 'La lettura delle notifiche*spenta*')
  Check 'nessuna notifica e mostrata' ((Pn "document.querySelectorAll('.notifs-item').length") -eq 0)
  Check 'il calendario c e ancora' ((Pn "document.querySelectorAll('.cal-day').length") -eq 42)
  Pn "document.querySelector('.notifs-btn').click()" | Out-Null
  Check 'Mostra le notifiche accende la lettura' (Wait-Until { (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='notif.read'})[0].value})") -eq $true } 5)
  Check 'e l elenco passa allo stato acceso (nessun messaggio di spento)' (Wait-Until { (Pn "document.querySelectorAll('.notifs-off').length") -eq 0 } 6)

  Write-Host 'B. tre notifiche mie nel pannello'
  Send 'n4-a' 'EosOS-N4-uno' 'primo corpo'
  Start-Sleep -Milliseconds 1200
  Send 'n4-b' 'EosOS-N4-due' 'secondo corpo'
  Start-Sleep -Milliseconds 1200
  Send 'n4-c' 'EosOS-N4-tre'
  Check 'compaiono nel gruppo Windows PowerShell entro la rilettura periodica' (Wait-Until { (@(My-Titles | Where-Object { $_ -like 'EosOS-N4*' })).Count -eq 3 } 12) ((My-Titles) -join ' / ')
  $t = @(My-Titles | Where-Object { $_ -like 'EosOS-N4*' })
  Check 'dalla piu recente: tre, due, uno' (($t.Count -eq 3) -and ($t[0] -eq 'EosOS-N4-tre') -and ($t[1] -eq 'EosOS-N4-due') -and ($t[2] -eq 'EosOS-N4-uno')) ($t -join ' / ')
  Check 'la piu recente mostra adesso e un corpo dove c e' ((Pn "(function(){var g=$myGroup;var it=Array.from(g.querySelectorAll('.notifs-item')).filter(function(i){return i.textContent.indexOf('EosOS-N4-due')>=0})[0];return it.querySelector('.notifs-body').textContent+'|'+it.querySelector('.notifs-ago').textContent})()") -eq 'secondo corpo|adesso')
  Check 'l intestazione ha il totale' ((Pn "document.querySelector('.notifs-title').textContent") -like 'Notifiche (*)')

  Write-Host 'C. togliere una notifica con la x'
  Pn "(function(){var g=$myGroup;Array.from(g.querySelectorAll('.notifs-item')).filter(function(i){return i.textContent.indexOf('EosOS-N4-due')>=0})[0].querySelector('.notifs-x').click()})()" | Out-Null
  Check 'sparisce subito dal pannello' (Wait-Until { -not ((My-Titles) -contains 'EosOS-N4-due') } 3)
  Start-Sleep -Milliseconds 5200
  Check 'e anche dopo la rilettura (e stata tolta davvero da Windows)' ((@(My-Titles | Where-Object { $_ -like 'EosOS-N4*' })).Count -eq 2 -and -not ((My-Titles) -contains 'EosOS-N4-due'))

  Write-Host 'D. cliccare una notifica apre la sua app e la toglie'
  Pn "(function(){var g=$myGroup;Array.from(g.querySelectorAll('.notifs-open')).filter(function(i){return i.textContent.indexOf('EosOS-N4-uno')>=0})[0].click()})()" | Out-Null
  Check 'Rust ha chiesto di aprire l app giusta (identificatore di PowerShell)' (Wait-Until { (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('notif_last_opened')") -eq $aumid } 5) ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('notif_last_opened')"))
  Check 'il pannello si chiude' (Wait-Until { -not (Panel-Open) } 4)
  [void](Open-Panel)
  Check 'e la notifica non c e piu' (Wait-Until { -not ((My-Titles) -contains 'EosOS-N4-uno') } 8)

  Write-Host 'E. togliere tutto il gruppo (solo se ci sono soltanto le mie)'
  Send 'n4-d' 'EosOS-N4-quattro' 'x'
  Send 'n4-e' 'EosOS-N4-cinque' 'y'
  Wait-Until { (@(My-Titles | Where-Object { $_ -like 'EosOS-N4*' })).Count -ge 3 } 12 | Out-Null
  $all = @(My-Titles)
  $onlyMine = ($all.Count -gt 0) -and (@($all | Where-Object { $_ -notlike 'EosOS-N4*' }).Count -eq 0)
  if ($onlyMine) {
    Pn "($myGroup).querySelector('.notifs-app .notifs-x').click()" | Out-Null
    Check 'il x del gruppo toglie tutte le notifiche dell app' (Wait-Until { (My-Titles).Count -eq 0 } 6)
    Start-Sleep -Milliseconds 5200
    Check 'e dopo la rilettura non ricompaiono' ((My-Titles).Count -eq 0)
  } else { Write-Host '  (nel gruppo PowerShell ci sono anche notifiche non mie: il x di gruppo non si prova)' -ForegroundColor Yellow }

  Write-Host 'F. spenta mentre il pannello e aperto'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.read',value:false})")
  Check 'entro la rilettura il pannello torna a dire che e spento' (Wait-Until { (Pn "document.querySelectorAll('.notifs-off').length") -eq 1 } 8)
  Check 'senza nessuna notifica mostrata' ((Pn "document.querySelectorAll('.notifs-item').length") -eq 0)
}
finally {
  & powershell -NoProfile -ExecutionPolicy Bypass -File $remove
  Remove-Item env:EOS_AUMID, env:EOS_TAG, env:EOS_T, env:EOS_L1, env:EOSOS_TEST_NO_OPEN -ErrorAction SilentlyContinue
  Remove-Item $toast, $remove -ErrorAction SilentlyContinue
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
