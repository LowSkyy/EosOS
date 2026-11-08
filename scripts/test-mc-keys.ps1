# Verifica M6 di Mission Control: Win+Tab e Alt+Tab con l'hook di tastiera. Finestre di prova: una
# Calcolatrice NUOVA e una console classica NUOVA (conhost, titolo "EosOS-Prova-B"; non si usa Windows
# Terminal, per non aggiungere una scheda a quello dell'utente). I tasti sono veri (keybd_event) ma vanno
# solo a finestre di prova o alla panoramica; nessun clic. Il file mission.json dell'utente viene salvato e
# ripristinato. I titoli delle finestre dell'utente non si stampano. Niente SendKeys.
#
# Uso: powershell -File scripts\test-mc-keys.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MK {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  public static string Cls(IntPtr h) { var sb = new StringBuilder(128); GetClassName(h, sb, 128); return sb.ToString(); }
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == title && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static void Down(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); }
  public static void Up(byte vk) { keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void Tap(byte vk) { Down(vk); Up(vk); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 40 }; $false }
function Overlays { @([MK]::Titled('EosOS Mission Control')) }
function IsOpen { (Overlays).Count -gt 0 }
function St { Inv 'mc_state' }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 10 }
function Find-New($before, [string]$namePart) { foreach ($h in [MK]::Visible()) { if ($before -notcontains $h -and ([MK]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function Focus([IntPtr]$h) {
  [void][MK]::SetForegroundWindow($h)
  if (-not (Wait-Until { [MK]::GetForegroundWindow() -eq $h } 1)) { [MK]::Down(0x12); [MK]::Up(0x12); [void][MK]::SetForegroundWindow($h); Wait-Until { [MK]::GetForegroundWindow() -eq $h } 2 | Out-Null }
  Start-Sleep -Milliseconds 300
}
function Pos($arr, $h) { for ($i = 0; $i -lt $arr.Count; $i++) { if ([int64]$arr[$i] -eq [int64]$h) { return $i } }; -1 }
function Close-Mc { [void](Inv 'mc_close'); Wait-Until { -not (IsOpen) } 3 | Out-Null; Start-Sleep -Milliseconds 400 }
function Release-All { foreach ($v in 0x12, 0xA4, 0xA0, 0x10, 0x5B, 0x11) { [MK]::Up([byte]$v) } }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = $null; if (Test-Path $mc) { $mcBackup = Get-Content $mc -Raw }; Remove-Item $mc -ErrorAction SilentlyContinue
$mcBad = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json.bad'; $badBackup = $null; if (Test-Path $mcBad) { $badBackup = Get-Content $mcBad -Raw; Remove-Item $mcBad }
$calc = [IntPtr]::Zero; $cons = [IntPtr]::Zero
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [MK]::Visible()
  Write-Host 'Preparazione: Calcolatrice e console di prova NUOVE'
  Start-Process 'calc.exe' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $calc -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; $calc = Find-New $before 'Calcolatrice' }
  Start-Process 'conhost.exe' -ArgumentList 'powershell.exe -NoProfile -NoExit -Command "$Host.UI.RawUI.WindowTitle=''EosOS-Prova-B''"' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $cons -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; $cons = Find-New $before 'EosOS-Prova-B' }
  Check 'finestre di prova create' (($calc -ne [IntPtr]::Zero) -and ($cons -ne [IntPtr]::Zero))
  if (($calc -eq [IntPtr]::Zero) -or ($cons -eq [IntPtr]::Zero)) { throw 'finestre di prova non trovate' }
  Start-Eos

  Write-Host 'A. l hook e installato e le sostituzioni sono attive di default'
  $s = St
  Check 'hook installato' ($s.keys.installed -eq $true)
  Check 'Alt+Tab e Win+Tab sostituiti di default' (($s.keys.alt_tab -eq $true) -and ($s.keys.win_tab -eq $true))
  Check 'nessuna sessione in corso' ($s.keys.session -eq $false)

  Write-Host 'B. Win+Tab apre e richiude la panoramica'
  Focus $calc
  [MK]::Down(0x5B); [MK]::Tap(0x09); Start-Sleep -Milliseconds 80; [MK]::Up(0x5B)
  Check 'Win+Tab apre' (Wait-Until { IsOpen } 5)
  Check 'a fine animazione e aperta' (Wait-Until { (St).phase -eq 'open' } 3)
  Check 'il menu Start non e comparso (la panoramica ha il primo piano)' ((Overlays) -contains [MK]::GetForegroundWindow())
  $sw = (St).keys.swallowed
  Check 'il Tab e stato mangiato (non e arrivato a Windows)' ($sw -ge 1) "mangiati $sw"
  [MK]::Down(0x5B); [MK]::Tap(0x09); Start-Sleep -Milliseconds 80; [MK]::Up(0x5B)
  Check 'Win+Tab di nuovo chiude' (Wait-Until { (St).phase -eq 'closed' } 4)
  Start-Sleep -Milliseconds 400
  Check 'dopo Win+Tab non e comparso il menu Start' ([MK]::Cls([MK]::GetForegroundWindow()) -ne 'XamlExplorerHostIslandWindow') ('primo piano classe ' + [MK]::Cls([MK]::GetForegroundWindow()))
  if ([MK]::Cls([MK]::GetForegroundWindow()) -eq 'XamlExplorerHostIslandWindow') { [MK]::Tap(0x1B); Start-Sleep -Milliseconds 400 }

  Write-Host 'C. Alt+Tab: scorre, Maiusc+Tab torna indietro, rilasciare Alt attiva'
  Focus $cons; Focus $calc   # ordine di uso: Calcolatrice, console, ...
  Check 'la Calcolatrice e in primo piano prima di Alt+Tab' ([MK]::GetForegroundWindow() -eq $calc) ("primo piano classe " + [MK]::Cls([MK]::GetForegroundWindow()) + " titolo-prova " + (([MK]::Title([MK]::GetForegroundWindow()) -like "*Calcolatrice*") -or ([MK]::Title([MK]::GetForegroundWindow()) -like "*EosOS-Prova*")) + "; calc classe " + [MK]::Cls($calc))
  [MK]::Down(0x12); Start-Sleep -Milliseconds 60; [MK]::Tap(0x09)
  Check 'Alt+Tab apre la panoramica' (Wait-Until { IsOpen } 5)
  Check 'la sessione e in corso' (Wait-Until { (St).keys.session -eq $true } 2)
  Wait-Until { (St).phase -eq 'open' } 3 | Out-Null
  $ord = @((St).order); Write-Host ('       ordine: Calcolatrice in posizione ' + (Pos $ord $calc) + ', console in posizione ' + (Pos $ord $cons) + ' su ' + $ord.Count)
  $first = (St).selected
  Check 'la prima pressione evidenzia la SECONDA finestra: la console di prova' ($first -eq [int64]$cons) "evidenziata $first, console $([int64]$cons)"
  $nWin = (St).windows
  Start-Sleep -Milliseconds 100; [MK]::Tap(0x09); Start-Sleep -Milliseconds 250
  $second = (St).selected
  if ($nWin -ge 3) { Check 'un altro Tab passa alla finestra dopo' (($second -ne $null) -and ($second -ne $first)) "evidenziata $second" }
  else { Check 'con due sole finestre il giro torna alla prima' ($second -eq [int64]$calc) "evidenziata $second" }
  [MK]::Down(0xA0); [MK]::Tap(0x09); [MK]::Up(0xA0); Start-Sleep -Milliseconds 250
  Check 'Maiusc+Tab torna alla precedente' ((St).selected -eq $first) ("evidenziata " + (St).selected)
  [MK]::Up(0x12)
  Check 'rilasciando Alt la panoramica si chiude' (Wait-Until { (St).phase -eq 'closed' } 4)
  Check 'la finestra evidenziata (la console) e ora in primo piano' (Wait-Until { [MK]::GetForegroundWindow() -eq $cons } 3)
  Check 'la sessione e finita' ((St).keys.session -eq $false)
  Check 'il rilascio di Alt non ha lasciato il menu della finestra aperto (il primo piano non e cambiato)' ([MK]::GetForegroundWindow() -eq $cons)

  Write-Host 'D. Alt+Tab veloce (un tocco): passa alla finestra precedente'
  Start-Sleep -Milliseconds 500
  Focus $calc; Focus $cons    # ora in primo piano la console, dietro la Calcolatrice
  [MK]::Down(0x12); [MK]::Tap(0x09); [MK]::Up(0x12)
  Check 'un tocco veloce porta in primo piano la Calcolatrice' (Wait-Until { [MK]::GetForegroundWindow() -eq $calc } 6)
  Check 'e la panoramica si chiude' (Wait-Until { (St).phase -eq 'closed' } 4)

  Write-Host 'E. Esc annulla senza cambiare finestra'
  Start-Sleep -Milliseconds 500
  Focus $cons; Focus $calc
  [MK]::Down(0x12); [MK]::Tap(0x09)
  Check 'si apre' (Wait-Until { (St).phase -eq 'open' } 5)
  [MK]::Tap(0x1B)
  Check 'Esc chiude' (Wait-Until { (St).phase -eq 'closed' } 4)
  Check 'la sessione e finita subito' ((St).keys.session -eq $false)
  [MK]::Up(0x12)
  Start-Sleep -Milliseconds 600
  Check 'rilasciare Alt dopo Esc non attiva la console' ([MK]::GetForegroundWindow() -ne $cons)

  Write-Host 'F. i tasti normali non si toccano'
  Focus $calc
  $sw0 = (St).keys.swallowed
  [MK]::Tap(0x09); [MK]::Tap(0x41); [MK]::Tap(0x1B); Start-Sleep -Milliseconds 300
  Check 'Tab, A, Esc da soli non vengono mangiati' ((St).keys.swallowed -eq $sw0)
  Check 'e non aprono niente' (-not (IsOpen))

  Write-Host 'G. passare a quelli di Windows dal Launcher (e tornare)'
  Check 'azione Alt+Tab accettata' ((Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-alttab'})") -eq 'done')
  $s = St
  Check 'Alt+Tab ora e quello di Windows, Win+Tab resta di EosOS' (($s.keys.alt_tab -eq $false) -and ($s.keys.win_tab -eq $true))
  Check 'la scelta e nel file' ((Get-Content $mc -Raw) -match '"windowsAltTab":\s*true')
  Check 'azione Win+Tab accettata' ((Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-wintab'})") -eq 'done')
  $s = St
  Check 'ora nessuna delle due e sostituita' (($s.keys.alt_tab -eq $false) -and ($s.keys.win_tab -eq $false))
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-alttab'})")
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-wintab'})")
  $s = St
  Check 'rimesse entrambe a EosOS' (($s.keys.alt_tab -eq $true) -and ($s.keys.win_tab -eq $true))
  # Con Alt+Tab di Windows scelto, Alt+Tab non e mangiato.
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-alttab'})")
  Focus $calc
  $sw0 = (St).keys.swallowed
  [MK]::Down(0x12); [MK]::Tap(0x09); Start-Sleep -Milliseconds 300; [MK]::Tap(0x1B); [MK]::Up(0x12); Start-Sleep -Milliseconds 400
  Check 'con quello di Windows scelto il Tab non viene mangiato' (((St).keys.swallowed -eq $sw0) -and (-not (IsOpen)))
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-alttab'})")

  Write-Host 'H. dopo il riavvio la scelta resta'
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-wintab'})")
  Cleanup-Eos; Start-Sleep 2; Start-Eos
  $s = St
  Check 'dopo il riavvio: Alt+Tab di EosOS, Win+Tab di Windows' (($s.keys.alt_tab -eq $true) -and ($s.keys.win_tab -eq $false))

  Write-Host 'I. il processo dell hook: si rilancia se muore, sparisce se muore EosOS'
  function Kids { @(Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -like '*--keyhook*' }) }
  Check 'c e un solo processo dell hook' (@(Kids).Count -eq 1) ("trovati " + @(Kids).Count)
  [void](Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'action:mc-wintab'})")  # Win+Tab di EosOS di nuovo attivo
  $k = @(Kids)[0]; Stop-Process -Id $k.ProcessId -Force
  Check 'ucciso l hook, EosOS se ne accorge' (Wait-Until { (St).keys.installed -eq $false } 4)
  Check 'e dopo qualche secondo ne lancia uno nuovo' (Wait-Until { ((St).keys.installed -eq $true) -and (@(Kids).Count -eq 1) } 10) ("installato " + (St).keys.installed + ", processi " + @(Kids).Count)
  Focus $calc
  [MK]::Down(0x5B); [MK]::Tap(0x09); Start-Sleep -Milliseconds 80; [MK]::Up(0x5B)
  Check 'Win+Tab funziona di nuovo con il nuovo hook' (Wait-Until { IsOpen } 5)
  Close-Mc
  # Se muore EosOS, l hook si chiude da solo (lo stdin si chiude).
  $main = Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notlike '*--keyhook*' -and $_.CommandLine -notlike '*--watchdog*' } | Select-Object -First 1
  Stop-Process -Id $main.ProcessId -Force
  Check 'senza EosOS l hook sparisce' (Wait-Until { @(Kids).Count -eq 0 } 6)
}
finally {
  Release-All
  try { if (IsOpen) { Close-Mc } } catch {}
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*EosOS-Prova-B*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  foreach ($h in @($calc)) { if ($h -ne [IntPtr]::Zero) { [void][MK]::PostMessage($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) } }
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($mcBackup) { Set-Content $mc $mcBackup -NoNewline -Encoding UTF8 }
  Remove-Item $mcBad -ErrorAction SilentlyContinue
  if ($badBackup) { Set-Content $mcBad $badBackup -NoNewline -Encoding UTF8 }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
