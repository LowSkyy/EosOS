# Centro notifiche, fase N1: prova di fattibilita. Chiede a Windows se EosOS (exe NON impacchettato) puo usare
# UserNotificationListener, e poi verifica fino in fondo con una notifica MIA: la manda una finestra di PowerShell
# (un toast con il titolo "EosOS-N1-prova"), si conta quante notifiche di quell'app ci sono prima e dopo, e la si toglie.
# Si leggono solo CONTEGGI e identificatori di app; mai titoli, testi o notifiche di altre app. La notifica di prova
# compare per un attimo come avviso sullo schermo e viene rimossa. Nessun consenso viene chiesto o concesso.
#
# Uso: powershell -File scripts\test-notif-probe.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Count([string]$aumid) { $a = $aumid.Replace('\', '\\'); Js "window.__TAURI_INTERNALS__.invoke('notif_count_for',{aumid:'$a'})" }

# L'identificatore dell'app PowerShell: le notifiche di questa prova risultano mandate da lei.
$aumid = '{1AC14E77-02E7-4E5D-B744-2EB1AE5198B7}\WindowsPowerShell\v1.0\powershell.exe'
try {
  Cleanup-Eos; Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. cosa dice Windows a un exe normale di EosOS'
  $p = Js "window.__TAURI_INTERNALS__.invoke('notif_probe')"
  Write-Host ("  risposta: identita di pacchetto=" + $p.has_package_identity + " (codice " + $p.package_code + "), listener=" + $p.listener.result + "/" + $p.listener.value + ", notifiche toast adesso=" + $p.toast_count + ", build=" + $p.build + ", conclusione=" + $p.verdict)
  Check 'EosOS non ha identita di pacchetto (e un exe normale)' ($p.has_package_identity -eq $false)
  Check 'la prova ha dato una conclusione' ($null -ne $p.verdict)

  if ($p.verdict -eq 'usable') {
    Check 'contare le notifiche funziona (nessun errore)' ($null -ne $p.toast_count -and $null -eq $p.count_error)
    Write-Host 'B. prova completa con una notifica MIA'
    $before = Count $aumid
    Check 'il conteggio per l app di prova e leggibile' ($null -ne $before) "prima: $before"
    # Il toast di prova (tag unico) dalla stessa app PowerShell.
    $ps = @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.Data.Xml.Dom.XmlDocument, Windows.Data.Xml.Dom.XmlDocument, ContentType = WindowsRuntime] | Out-Null
$xml = New-Object Windows.Data.Xml.Dom.XmlDocument
$xml.LoadXml('<toast><visual><binding template="ToastText01"><text id="1">EosOS-N1-prova</text></binding></visual></toast>')
$t = New-Object Windows.UI.Notifications.ToastNotification $xml
$t.Tag = 'eosos-n1'; $t.Group = 'eosos-n1'
[Windows.UI.Notifications.ToastNotificationManager]::CreateToastNotifier($env:EOS_AUMID).Show($t)
'@
    $env:EOS_AUMID = $aumid
    $f1 = Join-Path $env:TEMP 'eosos-n1-toast.ps1'; Set-Content $f1 $ps -Encoding UTF8   # in un file: PowerShell 5 toglie le virgolette dagli argomenti
    & powershell -NoProfile -ExecutionPolicy Bypass -File $f1
    Check 'dopo la notifica di prova il conteggio sale di uno' (Wait-Until { (Count $aumid) -eq ($before + 1) } 8) ("dopo: " + (Count $aumid))
    $psr = @'
[Windows.UI.Notifications.ToastNotificationManager, Windows.UI.Notifications, ContentType = WindowsRuntime] | Out-Null
[Windows.UI.Notifications.ToastNotificationManager]::History.Remove('eosos-n1', 'eosos-n1', $env:EOS_AUMID)
'@
    $f2 = Join-Path $env:TEMP 'eosos-n1-remove.ps1'; Set-Content $f2 $psr -Encoding UTF8
    & powershell -NoProfile -ExecutionPolicy Bypass -File $f2
    Remove-Item $f1, $f2 -ErrorAction SilentlyContinue
    Check 'tolta la notifica di prova il conteggio torna come prima' (Wait-Until { (Count $aumid) -eq $before } 8) ("ora: " + (Count $aumid))
  } else {
    Write-Host '  (il listener non e utilizzabile: la prova completa non si fa)'
  }
}
finally {
  Remove-Item env:EOS_AUMID -ErrorAction SilentlyContinue
  Cleanup-Eos
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
