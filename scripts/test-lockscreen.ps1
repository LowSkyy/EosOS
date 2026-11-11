# Verifica la schermata di blocco di EosOS (build con test-hooks): una finestra per monitor, sopra a tutto; i tasti che portano altrove si scartano
# (l'hook di tastiera lo dice); la password la verifica una funzione di prova (EOSOS_TEST_LOCK_PASSWORD, mai Windows); dopo 5 errori si aspetta anche
# con la password giusta; le altre finestre (Impostazioni, Launcher) non si aprono; con la schermata EosOS spenta si usa il blocco di Windows (nelle
# build di prova scrive solo la riga di prova); dopo 1 minuto passa il testimone a Windows; Windows Hello finto; controllo accessi.
# NON blocca davvero il PC di chi prova: le finestre le toglie lo script e il blocco vero di Windows in prova non parte mai.
#
# Uso: powershell -File scripts\test-lockscreen.ps1 [-Exe percorso\eosos.exe]   (dura circa 3 minuti; non toccare il mouse)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class LS {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] static extern long GetWindowLongPtr(IntPtr h, int i);
  [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
  // «titolo|sinistra|alto|destra|basso|sempreInPrimoPiano|visibile» di ogni finestra il cui titolo comincia per `prefix`.
  public static string[] Windows(string prefix) { var l = new List<string>(); EnumWindows((h, x) => { var sb = new StringBuilder(200); GetWindowText(h, sb, 200); var t = sb.ToString(); if (t.StartsWith(prefix)) { R r; GetWindowRect(h, out r); l.Add(t + "|" + r.L + "|" + r.T + "|" + r.Rt + "|" + r.B + "|" + (((GetWindowLongPtr(h, -20) & 8) != 0) ? 1 : 0) + "|" + (IsWindowVisible(h) ? 1 : 0)); } return true; }, IntPtr.Zero); return l.ToArray(); }
  public static bool Visible(string title) { foreach (var w in Windows(title)) { var p = w.Split('|'); if (p[6] == "1") return true; } return false; }
}
'@
[void][LS]::SetProcessDpiAwarenessContext([IntPtr](-4))
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Js([string]$view, [int]$idx, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$idx"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $o } } } finally { $ErrorActionPreference = $old } }
function Inv([string]$cmd, [string]$args2 = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$args2)" }
function Locks { @([LS]::Windows('EosOS Blocco ')) }
function Start-Eos([hashtable]$envs) {
  foreach ($k in $envs.Keys) { Set-Item "env:$k" $envs[$k] }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  foreach ($k in $envs.Keys) { Remove-Item "env:$k" -ErrorAction SilentlyContinue }
  Start-Sleep 14
}
function Do-Lock([string]$view = 'launcher') { [void](Js $view 0 "window.__TAURI_INTERNALS__.invoke('eosmenu_run',{id:'lock'})") }
function Unlock-Pw { [void](Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_password',{password:'prova-lock-123'})"); Wait-Until { (Locks).Count -eq 0 } 8 | Out-Null }
$monitors = @([System.Windows.Forms.Screen]::AllScreens)
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$log = Join-Path $env:TEMP 'eosos-actions.log'
try {
  Cleanup-Eos; Start-Sleep 2
  # Impostazioni di base: schermata EosOS accesa, nessun blocco da inattivita', Windows dopo 5 minuti.
  Remove-Item $mc -ErrorAction SilentlyContinue
  Start-Eos @{ EOSOS_TEST_LOCK_PASSWORD = 'prova-lock-123' }

  Write-Host 'A. impostazioni di base'
  $all = @(Inv 'settings_all' | ForEach-Object { $_ })
  $val = { param($k) ($all | Where-Object { $_.key -eq $k }).value }
  Check 'la schermata EosOS e accesa di base' ((& $val 'lock.eos') -eq $true)
  Check 'nessun blocco da inattivita di base' ((& $val 'lock.idle') -eq '0')
  Check 'dopo 5 minuti passa il testimone a Windows' ((& $val 'lock.windows') -eq '5')

  Write-Host 'B. blocco: una finestra per monitor, sopra a tutto'
  Do-Lock
  Check "compaiono $($monitors.Count) finestre di blocco (una per monitor)" (Wait-Until { (Locks).Count -eq $monitors.Count } 12) ("trovate " + (Locks).Count)
  $locks = Locks
  $covered = $true
  foreach ($m in $monitors) {
    $b = $m.Bounds
    $hit = $locks | Where-Object { $p = $_.Split('|'); ([Math]::Abs([int]$p[1] - $b.X) -le 2) -and ([Math]::Abs([int]$p[2] - $b.Y) -le 2) -and ([Math]::Abs([int]$p[3] - ($b.X + $b.Width)) -le 2) -and ([Math]::Abs([int]$p[4] - ($b.Y + $b.Height)) -le 2) }
    if (-not $hit) { $covered = $false; Write-Host "     nessuna finestra copre il monitor $($m.DeviceName) $b" }
  }
  Check 'ognuna copre tutto il suo monitor' $covered
  Check 'sono visibili e sempre in primo piano' (@($locks | Where-Object { $p = $_.Split('|'); $p[5] -eq '1' -and $p[6] -eq '1' }).Count -eq $monitors.Count)
  $keys = $null; Wait-Until { $script:keys = (Inv 'mc_state').keys; $script:keys.locked -eq $true } 4 | Out-Null
  Check 'l hook di tastiera sa che e bloccato (scarta Win, Alt+Tab, Alt+F4)' ($keys.locked -eq $true) ("installato " + $keys.installed)
  $info = Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_info')"
  Check 'la pagina riceve nome, lettera e modo password' ((-not [string]::IsNullOrEmpty($info.name)) -and ($info.initial.Length -ge 1) -and ($info.mode -eq 'password') -and ($info.hello -eq $false)) ("$($info.name) / $($info.mode)")
  Check 'l orologio e un HH:MM' ((Js 'lock' 0 "document.querySelector('.lock-time').textContent") -match '^\d\d:\d\d$')
  Check 'all inizio si vede solo l orologio (accesso nascosto)' ((Js 'lock' 0 "document.querySelector('.lock').getAttribute('data-awake')") -eq 'false')
  [void](Js 'lock' 0 "document.querySelector('.lock').dispatchEvent(new KeyboardEvent('keydown',{key:'a',bubbles:true}))")
  Check 'a un tasto compare l accesso' (Wait-Until { (Js 'lock' 0 "document.querySelector('.lock').getAttribute('data-awake')") -eq 'true' } 3)
  Check 'il campo password ha il focus' (Wait-Until { (Js 'lock' 0 "document.activeElement && document.activeElement.classList.contains('lock-field')") -eq $true } 3)

  Write-Host 'C. password giusta e sbagliata'
  $typeAndSubmit = { param($t) [void](Js 'lock' 0 "(function(){var i=document.querySelector('.lock-field'); var set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set; set.call(i,'$t'); i.dispatchEvent(new Event('input',{bubbles:true})); document.querySelector('.lock-form').requestSubmit(); return 'ok'})()") }
  & $typeAndSubmit 'sbagliata'
  Check 'una password sbagliata dice «Password errata» e svuota il campo' (Wait-Until { ((Js 'lock' 0 "document.querySelector('.lock-msg').textContent") -eq 'Password errata') -and ((Js 'lock' 0 "document.querySelector('.lock-field').value") -eq '') } 4)
  Check 'le finestre di blocco restano' ((Locks).Count -eq $monitors.Count)
  $wrong = { Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_password',{password:'altra-sbagliata'})" }
  $a2 = & $wrong; $a3 = & $wrong; $a4 = & $wrong
  Check 'fino al quarto errore nessuna attesa' (($a2.wait_secs -eq 0) -and ($a3.wait_secs -eq 0) -and ($a4.wait_secs -eq 0))
  $a5 = & $wrong
  Check 'al quinto errore l attesa e di 5 secondi' (($a5.ok -eq $false) -and ($a5.wait_secs -eq 5)) ("attesa " + $a5.wait_secs)
  $during = Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_password',{password:'prova-lock-123'})"
  Check 'durante l attesa anche la password GIUSTA viene rifiutata' (($during.ok -eq $false) -and ($during.wait_secs -ge 1)) ("attesa " + $during.wait_secs)
  Start-Sleep 6
  $right = Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_password',{password:'prova-lock-123'})"
  Check 'finita l attesa la password giusta sblocca' (Wait-Until { (Locks).Count -eq 0 } 8) ("windows rimaste " + (Locks).Count)
  Check 'l hook di tastiera torna libero' (Wait-Until { (Inv 'mc_state').keys.locked -eq $false } 4)

  Write-Host 'C2. mentre e bloccato le altre finestre non si aprono'
  Do-Lock
  [void](Wait-Until { (Locks).Count -eq $monitors.Count } 12)
  [void](Inv 'eosmenu_run' ",{id:'settings'}"); [void](Inv 'eosmenu_run' ",{id:'launchpad'}"); [void](Inv 'mc_open'); Start-Sleep 2
  Check 'le Impostazioni non si aprono' (-not [LS]::Visible('EosOS Impostazioni'))
  Check 'il Launchpad non si apre' (-not [LS]::Visible('EosOS Launcher'))
  Check 'Mission Control non si apre' (-not [LS]::Visible('EosOS Mission Control'))
  Unlock-Pw
  Check 'sbloccato, le Impostazioni tornano ad aprirsi' ((Inv 'eosmenu_run' ",{id:'settings'}") -eq $true -and (Wait-Until { [LS]::Visible('EosOS Impostazioni') } 6))
  [void](Inv 'settings_close'); Start-Sleep 1

  Write-Host 'D. schermata EosOS spenta: si usa il blocco di Windows'
  Remove-Item $log -ErrorAction SilentlyContinue
  $r = Inv 'settings_set' ",{key:'lock.eos',value:false}"
  Do-Lock; Start-Sleep 3
  Check 'nessuna finestra di blocco EosOS' ((Locks).Count -eq 0)
  Check 'chiesto il blocco di Windows (in prova solo la riga)' ((Test-Path $log) -and ((Get-Content $log -Raw) -match 'action:lock'))
  [void](Inv 'settings_set' ",{key:'lock.eos',value:true}")

  Write-Host 'E. dopo 1 minuto passa il testimone al blocco di Windows'
  [void](Inv 'settings_set' ",{key:'lock.windows',value:'1'}")
  Remove-Item $log -ErrorAction SilentlyContinue
  Do-Lock
  [void](Wait-Until { (Locks).Count -eq $monitors.Count } 12)
  Check 'e bloccato' ((Locks).Count -eq $monitors.Count)
  Check 'entro 75 secondi la schermata EosOS si toglie' (Wait-Until { (Locks).Count -eq 0 } 75) ("rimaste " + (Locks).Count)
  Check 'e chiede il blocco di Windows' ((Test-Path $log) -and ((Get-Content $log -Raw) -match 'action:lock'))
  Check 'l hook di tastiera torna libero' (Wait-Until { (Inv 'mc_state').keys.locked -eq $false } 4)
  [void](Inv 'settings_set' ",{key:'lock.windows',value:'5'}")

  Write-Host 'F. impostazioni non valide rifiutate'
  $bad = Inv 'settings_set' ",{key:'lock.idle',value:'7'}"
  Check 'minuti non ammessi rifiutati' ($bad.ok -eq $false) ($bad | ConvertTo-Json -Compress)

  Write-Host 'G. Windows Hello finto'
  Cleanup-Eos; Start-Sleep 2
  Start-Eos @{ EOSOS_TEST_LOCK_PASSWORD = 'prova-lock-123'; EOSOS_TEST_LOCK_HELLO = '1' }
  Do-Lock
  [void](Wait-Until { (Locks).Count -eq $monitors.Count } 12)
  $info = Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_info')"
  Check 'con Hello disponibile la pagina lo sa' ($info.hello -eq $true)
  Check 'il pulsante «Usa PIN o Windows Hello» c e' (Wait-Until { (Js 'lock' 0 "document.querySelectorAll('.lock-alt').length") -eq 1 } 3)
  [void](Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_hello')")
  Check 'la verifica di Hello sblocca' (Wait-Until { (Locks).Count -eq 0 } 8)

  Write-Host 'H. controllo accessi (le finestre di blocco si, le altre no)'
  Cleanup-Eos; Start-Sleep 2
  Start-Eos @{ EOSOS_TEST_LOCK_PASSWORD = 'prova-lock-123'; EOSOS_TEST_STRICT = '1' }
  # In modo severo eosmenu_run e' solo del pannello: si blocca da li' (come fa il menu EosOS).
  Check 'dal Launcher lock_info non da niente' ($null -eq (Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('lock_info').then(function(r){return r},function(e){return null})"))
  Do-Lock 'panel'
  [void](Wait-Until { (Locks).Count -eq $monitors.Count } 12)
  $x = Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('lock_try_password',{password:'prova-lock-123'}).then(function(r){return r},function(e){return {ok:false}})"
  Check 'dal Launcher non si puo sbloccare, nemmeno con la password giusta' (($x.ok -eq $false) -and ((Locks).Count -eq $monitors.Count))
  Check 'dalla finestra di blocco lock_info funziona' ($null -ne (Js 'lock' 0 "window.__TAURI_INTERNALS__.invoke('lock_info').then(function(r){return r},function(e){return null})"))
  Unlock-Pw
  Check 'e dalla finestra di blocco si sblocca' ((Locks).Count -eq 0)
}
finally {
  Cleanup-Eos
  if ($null -ne $mcBackup) { [System.IO.File]::WriteAllBytes($mc, $mcBackup) } else { Remove-Item $mc -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
