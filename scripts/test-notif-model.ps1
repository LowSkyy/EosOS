# Centro notifiche, fase N3: il modello delle notifiche con notifiche VERE di Windows, ma SOLO mie. Una finestra di PowerShell
# manda tre toast con titoli noti (uno con spazi e due righe di corpo, uno con un titolo lunghissimo); si leggono dal
# comando notif_list FILTRANDO nella pagina il solo gruppo dell'app PowerShell: le notifiche di altre app non escono mai
# dalla pagina e non si stampano. Prima si prova che con la lettura SPENTA (predefinita) non si legge niente. I toast di
# prova si tolgono alla fine. mission.json dell'utente viene salvato e ripristinato. Durante la prova compaiono per un
# attimo tre avvisi a schermo («EosOS-N3-...»).
#
# Uso: powershell -File scripts\test-notif-model.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }

$aumid = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
$aumidJs = $aumid.Replace('\', '\\').ToLower()
# Solo il gruppo dell'app PowerShell esce dalla pagina: {state, total, group} (mai le notifiche di altre app).
$listMine = "window.__TAURI_INTERNALS__.invoke('notif_list').then(function(r){var g=(r.groups||[]).filter(function(x){return x.app_id.toLowerCase()==='$aumidJs'})[0]||null;return JSON.stringify({state:r.state,message:r.message,groups:(r.groups||[]).length,total:r.total,mine:g,info:g?g.items.map(function(i){return {id:i.id,tl:i.title.length,bl:i.body.length,te:i.title.charCodeAt(i.title.length-1),sep:i.body.indexOf(String.fromCharCode(183))}}):[]})})"
function Mine { $r = Js $listMine; if ($r) { $r | ConvertFrom-Json } }

$toast = Join-Path $env:TEMP 'eosos-n3-toast.ps1'
Set-Content $toast @'
$Tag = $env:EOS_TAG; $Title = $env:EOS_T; $Line1 = $env:EOS_L1; $Line2 = $env:EOS_L2   # per variabili d'ambiente: un argomento vuoto non arriva a PowerShell 5
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$esc = { param($s) [System.Security.SecurityElement]::Escape($s) }
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$t = '<text>' + (& $esc $Title) + '</text>'
if ($Line1) { $t += '<text>' + (& $esc $Line1) + '</text>' }
if ($Line2) { $t += '<text>' + (& $esc $Line2) + '</text>' }
$xml.LoadXml('<toast><visual><binding template="ToastGeneric">' + $t + '</binding></visual></toast>')
$n = New-Object Windows.UI.Notifications.ToastNotification $xml
$n.Tag = $Tag; $n.Group = 'eosos-n3'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($env:EOS_AUMID).Show($n)
'@ -Encoding UTF8
$remove = Join-Path $env:TEMP 'eosos-n3-remove.ps1'
Set-Content $remove @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
foreach ($tag in 'n3-a', 'n3-b', 'n3-c') { try { [Windows.UI.Notifications.ToastNotificationManager]::History.Remove($tag, 'eosos-n3', $env:EOS_AUMID) } catch {} }
'@ -Encoding UTF8
function Send([string]$tag, [string]$title, [string]$l1 = '', [string]$l2 = '') { $env:EOS_TAG = $tag; $env:EOS_T = $title; $env:EOS_L1 = $l1; $env:EOS_L2 = $l2; & powershell -NoProfile -ExecutionPolicy Bypass -File $toast }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOS_AUMID = $aumid
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. con la lettura spenta (predefinita) non si legge niente'
  $off = Js "window.__TAURI_INTERNALS__.invoke('notif_list').then(function(r){return JSON.stringify({state:r.state,groups:r.groups.length,total:r.total})})" | ConvertFrom-Json
  Check 'la risposta e off, senza gruppi e senza totale' (($off.state -eq 'off') -and ($off.groups -eq 0) -and ($off.total -eq 0))
  Check 'togliere una notifica non si puo' ((Js "window.__TAURI_INTERNALS__.invoke('notif_dismiss',{id:1})") -eq $false)
  Check 'l impostazione Mostra le notifiche e spenta' (((Js "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='notif.read'})[0].value})")) -eq $false)

  Write-Host 'B. acceso: tre notifiche mie'
  $r = Js "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.read',value:true})"
  Check 'si accende dalle impostazioni' ($r.ok -eq $true)
  $before = Mine
  Check 'la lettura funziona' ($before.state -eq 'ok') $before.message
  $nBefore = if ($before.mine) { $before.mine.items.Count } else { 0 }
  Send 'n3-a' 'EosOS-N3-uno' 'primo corpo'
  Start-Sleep -Milliseconds 1200
  Send 'n3-b' '   EosOS-N3-due     con    spazi  ' 'riga 1' 'riga 2'
  Start-Sleep -Milliseconds 1200
  Send 'n3-c' ('EosOS-N3-tre-' + ('x' * 300)) ('corpo ' + ('y' * 400))
  $ok = Wait-Until { $m = Mine; $m.mine -and (@($m.mine.items | Where-Object { $_.title -like 'EosOS-N3*' }).Count -eq 3) } 10
  Check 'le tre notifiche di prova compaiono nel gruppo della loro app' $ok
  $m = Mine
  $mine = @($m.mine.items | Where-Object { $_.title -like 'EosOS-N3*' })
  $info = @($m.info | Where-Object { $mine.id -contains $_.id })   # lunghezze e codici di carattere letti nella pagina: la console di PowerShell 5 storpia gli accenti
  $infoBy = @{}; foreach ($i in $info) { $infoBy[[int]$i.id] = $i }
  Check 'il gruppo ha il nome dell app e l identificatore' ((-not [string]::IsNullOrWhiteSpace($m.mine.app_name)) -and ($m.mine.app_id.ToLower() -eq $aumid.ToLower())) ('nome: ' + $m.mine.app_name)
  Check 'dalla piu recente: tre, due, uno' (($mine.Count -eq 3) -and ($mine[0].title -like 'EosOS-N3-tre*') -and ($mine[1].title -like 'EosOS-N3-due*') -and ($mine[2].title -eq 'EosOS-N3-uno')) (($mine | ForEach-Object { $_.title.Substring(0, [math]::Min(14, $_.title.Length)) }) -join ' / ')
  Check 'il titolo con spazi e tabulazioni e ripulito' ($mine[1].title -eq 'EosOS-N3-due con spazi') $mine[1].title
  Check 'le righe di corpo si uniscono con il punto di mezzo' (($mine[1].body -like 'riga 1 * riga 2') -and ($infoBy[[int]$mine[1].id].sep -eq 7)) $mine[1].body
  Check 'il corpo semplice resta com e' ($mine[2].body -eq 'primo corpo')
  Check 'il titolo lunghissimo e tagliato a 100 caratteri con i puntini' (($infoBy[[int]$mine[0].id].tl -eq 100) -and ($infoBy[[int]$mine[0].id].te -eq 8230)) ("lunghezza " + $infoBy[[int]$mine[0].id].tl)
  Check 'il corpo lunghissimo e tagliato a 240' ($infoBy[[int]$mine[0].id].bl -eq 240) ("lunghezza " + $infoBy[[int]$mine[0].id].bl)
  Check 'la piu recente e appena arrivata: adesso' ($mine[0].ago -eq 'adesso') $mine[0].ago
  Check 'gli identificatori sono tutti diversi' ((@($mine | ForEach-Object { $_.id } | Select-Object -Unique).Count) -eq 3)
  Check 'il totale conta almeno le tre' ($m.total -ge $nBefore + 3) ("totale " + $m.total)

  Write-Host 'C. togliere una notifica'
  $idDue = $mine[1].id
  Check 'un id inventato e rifiutato' ((Js "window.__TAURI_INTERNALS__.invoke('notif_dismiss',{id:4000000000})") -eq $false)
  Check 'togliere la notifica due riesce' ((Js "window.__TAURI_INTERNALS__.invoke('notif_dismiss',{id:$idDue})") -eq $true)
  $gone = Wait-Until { $x = Mine; $x.mine -and (@($x.mine.items | Where-Object { $_.title -like 'EosOS-N3-due*' }).Count -eq 0) } 6
  Check 'sparisce dall elenco' $gone
  $left = @((Mine).mine.items | Where-Object { $_.title -like 'EosOS-N3*' })
  Check 'le altre due restano' (($left.Count -eq 2) -and ($left[0].title -like 'EosOS-N3-tre*') -and ($left[1].title -eq 'EosOS-N3-uno'))

  Write-Host 'D. di nuovo spento'
  [void](Js "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'notif.read',value:false})")
  $off = Js "window.__TAURI_INTERNALS__.invoke('notif_list').then(function(r){return JSON.stringify({state:r.state,groups:r.groups.length})})" | ConvertFrom-Json
  Check 'spento, non si legge piu niente' (($off.state -eq 'off') -and ($off.groups -eq 0))
  Check 'e non si toglie piu niente' ((Js "window.__TAURI_INTERNALS__.invoke('notif_dismiss',{id:$($left[0].id)})") -eq $false)
}
finally {
  & powershell -NoProfile -ExecutionPolicy Bypass -File $remove
  Remove-Item env:EOS_AUMID, env:EOS_TAG, env:EOS_T, env:EOS_L1, env:EOS_L2 -ErrorAction SilentlyContinue
  Remove-Item $toast, $remove -ErrorAction SilentlyContinue
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
