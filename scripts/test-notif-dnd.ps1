# Centro notifiche, fase N5: «Non disturbare» di EosOS (spegne solo la lettura nel pannello, non tocca Windows). Con notifiche
# VERE ma SOLO mie (tre toast mandati da una finestra di PowerShell con titoli «EosOS-N5-...»): si leggono dalla pagina del
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
$toast = Join-Path $env:TEMP 'eosos-n5-toast.ps1'
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
$remove = Join-Path $env:TEMP 'eosos-n5-remove.ps1'
Set-Content $remove @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
foreach ($tag in 'n5-a', 'n5-b', 'n5-c', 'n5-d', 'n5-e') { try { [Windows.UI.Notifications.ToastNotificationManager]::History.Remove($tag, 'eosos-n4', $env:EOS_AUMID) } catch {} }
'@ -Encoding UTF8
function Send([string]$tag, [string]$title, [string]$l1 = '') { $env:EOS_TAG = $tag; $env:EOS_T = $title; $env:EOS_L1 = $l1; & powershell -NoProfile -ExecutionPolicy Bypass -File $toast }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
function Ensure-Panel { if (-not (Panel-Open)) { [void](Open-Panel) } }
function Setting([string]$k) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='$k'})[0].value})" }
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOS_AUMID = $aumid; $env:EOSOS_TEST_NO_OPEN = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. predefinito'
  Check 'Non disturbare e spento di default' ((Setting 'notif.dnd') -eq $false)
  Check 'a lettura spenta il pulsante Non disturbare non c e (si vede solo Mostra le notifiche)' ((Open-Panel) -and ((Pn "Array.from(document.querySelectorAll('.notifs-btn')).map(function(b){return b.textContent}).join('|')") -eq 'Mostra le notifiche'))

  Write-Host 'B. con la lettura accesa'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.read',value:true})")
  Send 'n5-a' 'EosOS-N5-uno' 'corpo'; Start-Sleep 1; Ensure-Panel
  Check 'la mia notifica compare' (Wait-Until { (@(My-Titles | Where-Object { $_ -like 'EosOS-N5*' })).Count -ge 1 } 12)
  Check 'il pulsante Non disturbare c e, non premuto' ((Pn "(function(){var b=document.querySelector('.notifs-head .notifs-btn');return b.textContent+'|'+b.getAttribute('aria-pressed')})()") -eq 'Non disturbare|false')

  Write-Host 'C. Non disturbare acceso dal pannello'
  Ensure-Panel
  Pn "document.querySelector('.notifs-head .notifs-btn').click()" | Out-Null
  Check 'l impostazione risulta accesa' (Wait-Until { (Setting 'notif.dnd') -eq $true } 5)
  Check 'il pannello dice che e acceso' (Wait-Until { (Pn "(document.querySelector('.notifs [role=status]')||{textContent:''}).textContent") -like 'Non disturbare * acceso*' } 6)
  Check 'nessuna notifica e mostrata' ((Pn "document.querySelectorAll('.notifs-item').length") -eq 0)
  Check 'il pulsante diventa Riprendi, premuto' ((Pn "(function(){var b=document.querySelector('.notifs-head .notifs-btn');return b.textContent+'|'+b.getAttribute('aria-pressed')})()") -eq 'Riprendi|true')
  $r = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('notif_list')"
  Check 'Rust non legge: stato paused, nessun gruppo' (($r.state -eq 'paused') -and (@($r.groups).Count -eq 0))
  Check 'togliere/aprire una notifica e rifiutato' (((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('notif_dismiss',{id:1})") -eq $false) -and ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('notif_open',{id:1})") -eq $false))
  Check 'il calendario c e ancora' ((Pn "document.querySelectorAll('.cal-day').length") -eq 42)
  Start-Sleep -Milliseconds 5200
  Check 'dopo la rilettura periodica resta in pausa' ((Pn "document.querySelectorAll('.notifs-item').length") -eq 0)

  Write-Host 'D. Riprendi'
  Ensure-Panel
  Pn "document.querySelector('.notifs-head .notifs-btn').click()" | Out-Null
  Check 'l impostazione torna spenta' (Wait-Until { (Setting 'notif.dnd') -eq $false } 5)
  Check 'la mia notifica ricompare (Windows non e stato toccato)' (Wait-Until { (@(My-Titles | Where-Object { $_ -like 'EosOS-N5*' })).Count -ge 1 } 8)

  Ensure-Panel
  Write-Host 'E. acceso dalle Impostazioni mentre il pannello e aperto'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.dnd',value:true})")
  Check 'entro la rilettura il pannello va in pausa' (Wait-Until { (Pn "document.querySelectorAll('.notifs [role=status]').length") -eq 1 } 8)
  Check 'un valore non booleano e rifiutato' (((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.dnd',value:'forse'})").ok) -eq $false)
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
