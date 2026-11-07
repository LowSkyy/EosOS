# Verifica gli indicatori di sistema (I1) con una sorgente SIMULATA: batteria, rete e volume non si
# possono scaricare, staccare o cambiare a comando sul PC dell'utente. La build di prova legge lo
# stato da un file JSON (EOSOS_FAKE_INDICATORS) a ogni giro; qui si cambia il file e si osserva
# il comando `indicators_get` e l'evento `indicators-changed` dalla pagina del Launcher (via CDP).
# In coda una lettura VERA, in sola lettura. Non tocca volume, rete o batteria.
#
# Uso: powershell -File scripts\test-indicators.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$bytes = [System.IO.File]::ReadAllBytes($exe)
if (-not [System.Text.Encoding]::GetEncoding(28591).GetString($bytes).Contains('EOSOS_FAKE_INDICATORS')) { Write-Host 'Serve una build di prova (feature test-hooks).'; exit 2 }
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class IB {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr FindLike(string part) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().IndexOf(part, StringComparison.OrdinalIgnoreCase) >= 0) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $o = & node $cdp eval $expr; if ($o) { $o | ConvertFrom-Json } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Get-State { Js "window.__TAURI_INTERNALS__.invoke('indicators_get')" }
function Event-Count { [int](Js 'window.__ev.length') }
function BarJs([int]$i, [string]$expr) { $env:CDP_VIEW = 'bar'; $env:CDP_INDEX = "$i"; try { $o = & node $cdp eval $expr; if ($o) { $o | ConvertFrom-Json } } finally { Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [int]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }; $false }
function Write-Fake($json) { [System.IO.File]::WriteAllText($fake, $json, (New-Object System.Text.UTF8Encoding($false))) }
function Start-Eos([bool]$useFake) {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  if ($useFake) { $env:EOSOS_FAKE_INDICATORS = $fake } else { Remove-Item env:EOSOS_FAKE_INDICATORS -ErrorAction SilentlyContinue }
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_FAKE_INDICATORS -ErrorAction SilentlyContinue
  Start-Sleep 9
}
$fake = Join-Path $env:TEMP ('eosos-fake-indicators-' + (Get-Random -Maximum 99999) + '.json')
try {
  Cleanup-Eos; Start-Sleep 1
  Write-Host 'A. stato iniziale simulato'
  Write-Fake '{"battery":{"percent":85,"charging":true,"plugged":true},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4},"volume":{"percent":40,"muted":false}}'
  Start-Eos $true
  $ok = Wait-Until { $s = Get-State; $s -and $s.volume }
  Check 'indicators_get risponde con lo stato' $ok
  $s = Get-State
  Check 'batteria: percentuale sempre nel testo' ($s.battery.label -eq '85%') $s.battery.title
  Check 'batteria: in carica, icona di ricarica' (($s.battery.charging -eq $true) -and ($s.battery.icon -eq 'battery-charging'))
  Check 'rete: Wi-Fi con nome e barre' (($s.network.kind -eq 'wifi') -and ($s.network.title -eq 'Wi-Fi: Casa') -and ($s.network.icon -eq 'wifi-3')) $s.network.title
  Check 'volume: 40%, icona media' (($s.volume.percent -eq 40) -and ($s.volume.icon -eq 'volume-2') -and ($s.volume.title -eq 'Volume 40%'))

  Write-Host 'B. evento solo sui cambi'
  [void](Js "(async()=>{window.__ev=[];var h=window.__TAURI_INTERNALS__.transformCallback(function(e){window.__ev.push(e.payload)});await window.__TAURI_INTERNALS__.invoke('plugin:event|listen',{event:'indicators-changed',target:{kind:'Any'},handler:h});return 1})()")
  Start-Sleep -Milliseconds 500
  Check 'nessun evento senza cambi (4 s di attesa)' ((Start-Sleep 4) -eq $null -and (Event-Count) -eq 0)
  Write-Fake '{"battery":{"percent":19,"plugged":false,"minutes_left":135},"network":{"kind":"wifi","online":"limited","name":"Ospiti","bars":2},"volume":{"percent":0,"muted":true}}'
  Check 'un cambio produce un evento' (Wait-Until { (Event-Count) -ge 1 })
  Check 'un solo evento per il cambio' ((Start-Sleep 3) -eq $null -and (Event-Count) -eq 1) ("eventi: " + (Event-Count))
  $p = Js 'window.__ev[0]'
  Check 'evento: batteria scarica con tempo residuo' (($p.battery.label -eq '19%') -and ($p.battery.icon -eq 'battery-low') -and ($p.battery.title -like 'Batteria 19%*2 h 15 min rimanenti')) $p.battery.title
  Check 'evento: Wi-Fi senza Internet' (($p.network.icon -eq 'wifi-alert') -and ($p.network.title -like 'Wi-Fi: Ospiti*senza Internet'))
  Check 'evento: audio disattivato' (($p.volume.icon -eq 'volume-muted') -and ($p.volume.title -eq 'Audio disattivato'))
  $s = Get-State
  Check 'indicators_get coincide con l evento' (($s.battery.percent -eq 19) -and ($s.volume.muted -eq $true))

  Write-Host 'C. hardware che sparisce e ricompare'
  Write-Fake '{"network":{"kind":"none","online":"none"}}'
  Check 'senza batteria e senza audio: i campi diventano null' (Wait-Until { $x = Get-State; ($null -eq $x.battery) -and ($null -eq $x.volume) })
  $s = Get-State
  Check 'rete scollegata: icona e testo' (($s.network.icon -eq 'net-off') -and ($s.network.title -eq 'Nessuna connessione'))
  Check 'evento anche per la scomparsa' ((Event-Count) -eq 2) ("eventi: " + (Event-Count))

  Write-Host 'D. file illeggibile: si tiene lo stato di prima'
  Write-Fake 'questo non e json {'
  Start-Sleep 4
  $s = Get-State
  Check 'stato invariato e nessun evento' (($s.network.icon -eq 'net-off') -and (Event-Count) -eq 2)
  Write-Fake '{"volume":{"percent":250,"muted":false},"battery":{"percent":150}}'
  Check 'valori fuori scala vengono limitati a 100' (Wait-Until { $x = Get-State; $x.volume -and $x.volume.percent -eq 100 })
  Check 'anche la batteria' ((Get-State).battery.percent -eq 100)


  Write-Host 'F. indicatori nella barra (una pagina per monitor)'
  Write-Fake '{"battery":{"percent":85,"charging":true,"plugged":true},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4},"volume":{"percent":40,"muted":false}}'
  Check 'la barra mostra i tre indicatori' (Wait-Until { (BarJs 0 "document.querySelectorAll('.indicator').length") -eq 3 })
  $order = BarJs 0 "Array.from(document.querySelectorAll('.indicator')).map(function(e){return e.dataset.key}).join(',')"
  Check 'ordine: batteria, rete, audio' ($order -eq 'battery,network,volume') $order
  Check 'percentuale della batteria sempre visibile' ((BarJs 0 "document.querySelector('.indicator[data-key=battery] .indicator-label').textContent") -eq '85%')
  Check 'ogni indicatore ha un suggerimento e un nome per i lettori di schermo' ((BarJs 0 "Array.from(document.querySelectorAll('.indicator')).every(function(e){return e.title && e.getAttribute('aria-label') === e.title})") -eq $true)
  Check 'nessun indicatore in allarme' ((BarJs 0 "document.querySelectorAll('.indicator[data-alert=true]').length") -eq 0)
  $h = [double](BarJs 0 "document.querySelector('.menubar').getBoundingClientRect().height")
  Check 'l altezza della barra non cambia (28 px CSS)' ([Math]::Abs($h - 28) -lt 1) "($h)"
  $second = BarJs 1 "document.querySelectorAll('.indicator').length"
  if ($null -ne $second) { Check 'anche la barra del secondo monitor li mostra' ($second -eq 3) } else { Write-Host '  (un solo monitor: una sola barra)' }
  if ($env:INDICATORS_SHOT) {
    $env:CDP_VIEW = 'bar'; & node $cdp shot ($env:INDICATORS_SHOT + '-dark.png') | Out-Null
    [void](BarJs 0 "(document.documentElement.setAttribute('data-theme','light'),1)"); Start-Sleep -Milliseconds 400
    $env:CDP_VIEW = 'bar'; & node $cdp shot ($env:INDICATORS_SHOT + '-light.png') | Out-Null
    [void](BarJs 0 "(document.documentElement.setAttribute('data-theme','dark'),1)")
  }

  Write-Fake '{"battery":{"percent":5,"plugged":false,"minutes_left":20},"network":{"kind":"wifi","online":"limited","name":"Ospiti","bars":2},"volume":{"percent":0,"muted":true}}'
  Check 'batteria quasi scarica: in allarme' (Wait-Until { (BarJs 0 "document.querySelector('.indicator[data-key=battery]').dataset.alert") -eq 'true' })
  Check 'rete senza Internet: in allarme' ((BarJs 0 "document.querySelector('.indicator[data-key=network]').dataset.alert") -eq 'true')
  Check 'audio disattivato: non e un allarme' ((BarJs 0 "document.querySelector('.indicator[data-key=volume]').dataset.alert") -eq 'false')
  Check 'la percentuale scende a 5%' ((BarJs 0 "document.querySelector('.indicator[data-key=battery] .indicator-label').textContent") -eq '5%')
  if ($env:INDICATORS_SHOT) { $env:CDP_VIEW = 'bar'; & node $cdp shot ($env:INDICATORS_SHOT + '-alert.png') | Out-Null }

  Write-Fake '{"network":{"kind":"ethernet","online":"internet","name":"Ethernet"},"volume":{"percent":100,"muted":false}}'
  Check 'senza batteria (come questo PC): compaiono solo rete e audio' (Wait-Until { (BarJs 0 "Array.from(document.querySelectorAll('.indicator')).map(function(e){return e.dataset.key}).join(',')") -eq 'network,volume' })
  Write-Fake '{}'
  Check 'senza nulla: il blocco indicatori sparisce' (Wait-Until { (BarJs 0 "document.querySelectorAll('.indicators').length") -eq 0 })
  Write-Fake '{"battery":{"percent":50},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":5},"volume":{"percent":50,"muted":false}}'
  Check 'ricompaiono quando tornano' (Wait-Until { (BarJs 0 "document.querySelectorAll('.indicator').length") -eq 3 })

  Write-Host 'G. clic vero sull indicatore di rete apre le Impostazioni giuste'
  $settingsBefore = [IB]::FindLike('Impostazioni')
  $rect = BarJs 0 "(function(){var r=document.querySelector('.indicator[data-key=network]').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2, window.devicePixelRatio, window.screenX, window.screenY]})()"
  $bars = [IB]::Titled('EosOS')
  # La barra del monitor principale: quella che contiene l'origine dello schermo principale.
  Add-Type -AssemblyName System.Windows.Forms
  $pb = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $barWin = [IntPtr]::Zero
  foreach ($b in $bars) { $r = New-Object IB+R; [void][IB]::GetWindowRect($b, [ref]$r); if ($r.L -eq $pb.X -and $r.T -eq $pb.Y) { $barWin = $b } }
  if ($barWin -ne [IntPtr]::Zero -and $rect) {
    $r = New-Object IB+R; [void][IB]::GetWindowRect($barWin, [ref]$r)
    $px = [int]($r.L + $rect[0] * $rect[2]); $py = [int]($r.T + $rect[1] * $rect[2])
    [void][IB]::SetCursorPos($px, $py); Start-Sleep -Milliseconds 400
    $under = [IB]::Root($px, $py)
    Check 'sotto il cursore c e la barra di EosOS (si clicca solo se e vero)' ($under -eq $barWin)
    if ($under -eq $barWin -and $settingsBefore -eq [IntPtr]::Zero) {
      [IB]::Click(); Start-Sleep -Seconds 3
      $sw = [IB]::FindLike('Impostazioni')
      Check 'si apre la finestra Impostazioni' ($sw -ne [IntPtr]::Zero)
      if ($sw -ne [IntPtr]::Zero) { [void][IB]::PostMessage($sw, 0x10, [IntPtr]::Zero, [IntPtr]::Zero); Start-Sleep -Milliseconds 800 }
    } elseif ($settingsBefore -ne [IntPtr]::Zero) { Write-Host '  (Impostazioni era gia aperta: clic non provato)' -ForegroundColor Yellow }
    [void][IB]::SetCursorPos($pb.X + [int]($pb.Width / 2), $pb.Y + [int]($pb.Height / 2))
  } else { Check 'barra del monitor principale trovata' $false }

  Write-Host 'E. lettura VERA (sola lettura)'
  Cleanup-Eos; Start-Sleep 2
  Start-Eos $false
  Check 'indicators_get risponde' (Wait-Until { $x = Get-State; $x -and $x.network })
  $r = Get-State
  Write-Host ("  stato reale: batteria=" + $(if ($r.battery) { $r.battery.title } else { 'assente' }) + " | rete=" + $r.network.title + " | volume=" + $(if ($r.volume) { $r.volume.title } else { 'assente' }))
  Check 'la rete e sempre presente' ($null -ne $r.network -and $r.network.title)
  if ($r.volume) { Check 'volume reale tra 0 e 100' (($r.volume.percent -ge 0) -and ($r.volume.percent -le 100)) }
  if ($r.battery) { Check 'batteria reale: etichetta con percentuale' ($r.battery.label -match '^\d{1,3}%$') }
}
finally {
  Cleanup-Eos
  Remove-Item $fake -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
