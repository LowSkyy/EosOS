# Verifica M3 di Mission Control: animazione, scorciatoia Ctrl+Alt+Su, angolo attivo. Bersagli di TUTTI i clic
# e della pressione del mouse: solo una Calcolatrice di prova NUOVA, e solo dopo aver verificato che sotto il
# cursore ci sia lei. Tasti veri solo con una finestra di prova in primo piano (Ctrl+Alt+Su non fa danni li').
# Il file mission.json dell'utente viene salvato e ripristinato. Niente SendKeys. Le finestre dell'utente
# compaiono nella panoramica come in uso reale ma non si toccano e i loro titoli non si stampano.
#
# Uso: powershell -File scripts\test-mc-trigger.ps1 [-Exe percorso\eosos.exe] [-SkipConflict]
param([string]$Exe, [switch]$SkipConflict)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MT {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static string Cls(IntPtr h) { var sb = new StringBuilder(128); GetClassName(h, sb, 128); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == title && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr MessageBoxEosOS() { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h) == "EosOS" && Cls(h) == "#32770") { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void LeftDown() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); }
  public static void LeftUp() { mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void CtrlAltUp() { keybd_event(0x11, 0, 0, UIntPtr.Zero); keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x26, 0, 1, UIntPtr.Zero); keybd_event(0x26, 0, 3, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); keybd_event(0x11, 0, 2, UIntPtr.Zero); }
  public static void AltSpace() { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 40 }; $false }
function Overlays { @([MT]::Titled('EosOS Mission Control')) }
function IsOpen { (Overlays).Count -gt 0 }
function St { Inv 'mc_state' }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 10 }
function Find-New($before, [string]$namePart) { foreach ($h in [MT]::Visible()) { if ($before -notcontains $h -and ([MT]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function Esc-If-Open { if (IsOpen) { $fg = [MT]::GetForegroundWindow(); if ((Overlays) -contains $fg) { [MT]::Key(0x1B) }; Wait-Until { -not (IsOpen) } 3 | Out-Null } }
function Close-Mc { [void](Inv 'mc_close'); Wait-Until { -not (IsOpen) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$mcBackup = $null; if (Test-Path $mc) { $mcBackup = Get-Content $mc -Raw }; Remove-Item $mc -ErrorAction SilentlyContinue
$mcBad = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json.bad'; $badBackup = $null; if (Test-Path $mcBad) { $badBackup = Get-Content $mcBad -Raw; Remove-Item $mcBad }
$pb = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$calc = [IntPtr]::Zero; $holder = $null; $holderFile = Join-Path $env:TEMP 'eosos-mc-holder.ps1'
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [MT]::Visible()
  Write-Host 'Preparazione: una Calcolatrice NUOVA come finestra di prova'
  Start-Process 'calc.exe' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $calc -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; $calc = Find-New $before 'Calcolatrice' }
  Check 'finestra di prova creata' ($calc -ne [IntPtr]::Zero)
  if ($calc -eq [IntPtr]::Zero) { throw 'Calcolatrice di prova non trovata' }
  Start-Eos

  Write-Host 'A. animazione di ingresso'
  # Le fasi si osservano DENTRO la pagina in un ciclo stretto (chiamare un processo esterno costa ~150 ms, piu' dell'animazione).
  $phases = Js 'launcher' 0 "(async function(){await window.__TAURI_INTERNALS__.invoke('mc_open');var seen=[];var t0=performance.now();while(performance.now()-t0<600){var s=await window.__TAURI_INTERNALS__.invoke('mc_state');if(seen.indexOf(s.phase)<0)seen.push(s.phase)}return seen.join(',')})()"
  Check 'le fasi viste sono: in apertura, poi aperta' (($phases -like 'opening*') -and ($phases -like '*open')) "fasi: $phases"
  Check 'passa ad "aperta"' (Wait-Until { (St).phase -eq 'open' } 3)
  $s = St
  Check 'l animazione ha fatto almeno 8 fotogrammi' ($s.frames -ge 8) ("fotogrammi " + $s.frames)
  Check 'e e durata circa 220 ms (non scatta ne trascina)' (($s.anim_ms -ge 200) -and ($s.anim_ms -le 450)) ("durata " + $s.anim_ms + " ms")
  Check 'miniature registrate e finestre elencate' (($s.thumbs -ge 1) -and ($s.windows -ge 1))

  Write-Host 'B. animazione di uscita (Esc)'
  $fg = [MT]::GetForegroundWindow()
  Check 'la panoramica e in primo piano' ((Overlays) -contains $fg)
  # Esc chiude: la fase "in chiusura" si osserva con lo stesso metodo, partendo da una chiusura chiesta dalla pagina.
  Close-Mc
  [void](Inv 'mc_open'); Wait-Until { (St).phase -eq 'open' } 3 | Out-Null
  $phases = Js 'launcher' 0 "(async function(){await window.__TAURI_INTERNALS__.invoke('mc_close');var seen=[];var t0=performance.now();while(performance.now()-t0<600){var s=await window.__TAURI_INTERNALS__.invoke('mc_state');if(seen.indexOf(s.phase)<0)seen.push(s.phase)}return seen.join(',')})()"
  Check 'le fasi viste sono: in chiusura, poi chiusa' (($phases -like 'closing*') -and ($phases -like '*closed')) "fasi: $phases"
  Check 'passa a "chiusa"' (Wait-Until { (St).phase -eq 'closed' } 3)
  # Esc con la tastiera vera: stessa strada.
  [void](Inv 'mc_open'); Wait-Until { (St).phase -eq 'open' } 3 | Out-Null
  $fg = [MT]::GetForegroundWindow()
  if ((Overlays) -contains $fg) { [MT]::Key(0x1B) }
  Check 'Esc con la tastiera vera chiude' (Wait-Until { (St).phase -eq 'closed' } 3)
  $s = St
  Check 'animazione inversa: fotogrammi e durata circa 180 ms' (($s.frames -ge 6) -and ($s.anim_ms -ge 160) -and ($s.anim_ms -le 400)) ("fotogrammi " + $s.frames + ", durata " + $s.anim_ms + " ms")
  Check 'finestre nascoste e miniature liberate' ((-not (IsOpen)) -and ($s.thumbs -eq 0))
  Start-Sleep -Milliseconds 300

  Write-Host 'C. chiusura veloce durante l apertura: nessun blocco'
  [void](Inv 'mc_open'); [void](Inv 'mc_close')
  $ok = Wait-Until { (St).phase -eq 'closed' -and -not (IsOpen) } 3
  Check 'chiusa subito, senza restare a meta' $ok
  Check 'si puo riaprire e richiudere' ((Inv 'mc_open') -eq $null -and (Wait-Until { (St).phase -eq 'open' } 3))
  Close-Mc
  Check 'chiusa' ((St).phase -eq 'closed')

  Write-Host 'D. scorciatoia Ctrl+Alt+Su'
  [void][MT]::SetForegroundWindow($calc); Start-Sleep -Milliseconds 500
  Check 'la finestra di prova e in primo piano (i tasti vanno a lei se la scorciatoia non c e)' ([MT]::GetForegroundWindow() -eq $calc)
  [MT]::CtrlAltUp()
  Check 'la scorciatoia apre' (Wait-Until { IsOpen } 4)
  Check 'a fine animazione e aperta' (Wait-Until { (St).phase -eq 'open' } 3)
  [MT]::CtrlAltUp()
  Check 'la stessa scorciatoia chiude' (Wait-Until { (St).phase -eq 'closed' } 4)
  Close-Mc

  Write-Host 'E. angolo attivo: spento di default'
  [void][MT]::SetCursorPos($pb.X, $pb.Y); Start-Sleep -Milliseconds 1200
  Check 'fermo nell angolo 1,2 s: non succede nulla' (-not (IsOpen))
  Check 'lo stato dice angolo spento e il file non esiste' (((St).corner -eq $false) -and (-not (Test-Path $mc)))
  [void][MT]::SetCursorPos($pb.X + 600, $pb.Y + 400)

  Write-Host 'F. lo si accende dal Launcher'
  [void][MT]::SetCursorPos($pb.X + 400, $pb.Y + 400)
  [MT]::AltSpace(); Start-Sleep -Milliseconds 1000
  $lw = [MT]::Titled('EosOS Launcher'); $lf = ($lw.Count -gt 0 -and [MT]::GetForegroundWindow() -eq $lw[0])
  Check 'Launcher aperto e a fuoco' $lf
  if ($lf) {
    foreach ($c in 'angolo'.ToCharArray()) { [MT]::Key([byte][char]$c.ToString().ToUpper()); Start-Sleep -Milliseconds 70 }
    Start-Sleep -Milliseconds 900
    [MT]::Key(0x0D); Start-Sleep -Milliseconds 800
  }
  Check 'angolo attivo acceso' (Wait-Until { (St).corner -eq $true } 3)
  Check 'scelta salvata in mission.json' ((Test-Path $mc) -and ((Get-Content $mc -Raw) -match '"hotCorner":\s*true'))

  Write-Host 'G. angolo attivo: scatta dopo la permanenza, una volta sola'
  [void][MT]::SetCursorPos($pb.X + 600, $pb.Y + 400); Start-Sleep -Milliseconds 400
  [void][MT]::SetCursorPos($pb.X, $pb.Y); $t = Get-Date
  Start-Sleep -Milliseconds 250
  Check 'dopo 250 ms ancora no' (-not (IsOpen))
  $fired = Wait-Until { IsOpen } 2.5
  $dt = ((Get-Date) - $t).TotalMilliseconds
  Check 'scatta' $fired
  Check 'dopo circa mezzo secondo (non subito, non tardi)' (($dt -ge 450) -and ($dt -le 1600)) ("dopo " + [Math]::Round($dt) + " ms")
  Check 'a fine animazione e aperta' (Wait-Until { (St).phase -eq 'open' } 3)
  $fg = [MT]::GetForegroundWindow()
  if ((Overlays) -contains $fg) { [MT]::Key(0x1B) }
  Check 'Esc la chiude' (Wait-Until { -not (IsOpen) } 3)
  Start-Sleep -Milliseconds 1500
  Check 'restando nell angolo non riscatta' (-not (IsOpen))
  [void][MT]::SetCursorPos($pb.X + 600, $pb.Y + 400); Start-Sleep -Milliseconds 300
  [void][MT]::SetCursorPos($pb.X, $pb.Y)
  Check 'uscendo e rientrando scatta di nuovo' (Wait-Until { IsOpen } 2.5)
  Wait-Until { (St).phase -eq 'open' } 3 | Out-Null
  Close-Mc
  [void][MT]::SetCursorPos($pb.X + 600, $pb.Y + 400); Start-Sleep -Milliseconds 400

  Write-Host 'H. angolo attivo: con un pulsante premuto non scatta (bersaglio: la finestra di prova)'
  [void][MT]::SetForegroundWindow($calc); Start-Sleep -Milliseconds 400
  $r = New-Object MT+R; [void][MT]::GetWindowRect($calc, [ref]$r)
  $cx = [int](($r.L + $r.Rt) / 2); $cy = [int]($r.T + 90)   # il display dei risultati: nessuna azione al clic
  [void][MT]::SetCursorPos($cx, $cy); Start-Sleep -Milliseconds 300
  $onCalc = ([MT]::Root($cx, $cy) -eq $calc)
  Check 'sotto il cursore c e la Calcolatrice di prova (si preme il pulsante solo se e vero)' $onCalc
  if ($onCalc) {
    [MT]::LeftDown(); Start-Sleep -Milliseconds 150
    [void][MT]::SetCursorPos($pb.X, $pb.Y); Start-Sleep -Milliseconds 1300
    Check 'con il pulsante premuto non scatta' (-not (IsOpen))
    [MT]::LeftUp()
    Check 'rilasciato dentro l angolo scatta dopo la permanenza' (Wait-Until { IsOpen } 2.5)
    Wait-Until { (St).phase -eq 'open' } 3 | Out-Null
    Close-Mc
  }
  [void][MT]::SetCursorPos($pb.X + 600, $pb.Y + 400)

  Write-Host 'I. lo si spegne di nuovo e la scelta sopravvive al riavvio'
  [MT]::AltSpace(); Start-Sleep -Milliseconds 1000
  $lw = [MT]::Titled('EosOS Launcher'); $lf = ($lw.Count -gt 0 -and [MT]::GetForegroundWindow() -eq $lw[0])
  if ($lf) { foreach ($c in 'angolo'.ToCharArray()) { [MT]::Key([byte][char]$c.ToString().ToUpper()); Start-Sleep -Milliseconds 70 }; Start-Sleep -Milliseconds 900; [MT]::Key(0x0D); Start-Sleep -Milliseconds 800 }
  Check 'spento' (Wait-Until { (St).corner -eq $false } 3)
  Check 'salvato' ((Get-Content $mc -Raw) -match '"hotCorner":\s*false')
  Cleanup-Eos; Start-Sleep 2
  [System.IO.File]::WriteAllText($mc, '{"version":1,"hotCorner":true}', (New-Object System.Text.UTF8Encoding($false)))
  Start-Eos
  Check 'dopo il riavvio l angolo e acceso come da file' ((St).corner -eq $true)
  Cleanup-Eos; Start-Sleep 2
  [System.IO.File]::WriteAllText($mc, 'questo non e json {', (New-Object System.Text.UTF8Encoding($false)))
  Start-Eos
  Check 'con il file illeggibile si parte con l angolo spento' ((St).corner -eq $false)
  Check 'e il file rotto non viene toccato' ((Get-Content $mc -Raw) -like 'questo non e json*')

  if (-not $SkipConflict) {
    Write-Host 'J. scorciatoia gia occupata da un altro programma: avviso e nuovi tentativi'
    Cleanup-Eos; Start-Sleep 2
    Remove-Item $mc -ErrorAction SilentlyContinue
    Set-Content -Path $holderFile -Encoding ASCII -Value @'
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class HH {
  [StructLayout(LayoutKind.Sequential)] public struct M { public IntPtr h; public uint m; public UIntPtr w; public IntPtr l; public uint t; public int x, y; }
  [DllImport("user32.dll")] public static extern bool RegisterHotKey(IntPtr h, int id, uint mod, uint vk);
  [DllImport("user32.dll")] public static extern int GetMessage(out M m, IntPtr h, uint a, uint b);
  public static void Run() { M m; RegisterHotKey(IntPtr.Zero, 1, 0x4003, 0x26); while (GetMessage(out m, IntPtr.Zero, 0, 0) > 0) {} }
}
"@
[HH]::Run()
'@
    $holder = Start-Process powershell -ArgumentList '-NoProfile', '-File', $holderFile -PassThru -WindowStyle Hidden
    Start-Sleep 2
    Start-Eos
    $box = [IntPtr]::Zero
    $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 10 -and $box -eq [IntPtr]::Zero) { $box = [MT]::MessageBoxEosOS(); Start-Sleep -Milliseconds 300 }
    Check 'compare l avviso di conflitto' ($box -ne [IntPtr]::Zero)
    if ($box -ne [IntPtr]::Zero) { [void][MT]::PostMessage($box, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
    [void][MT]::SetForegroundWindow($calc); Start-Sleep -Milliseconds 400
    [MT]::CtrlAltUp(); Start-Sleep -Milliseconds 1200
    Check 'finche e occupata la scorciatoia non apre' (-not (IsOpen))
    Stop-Process -Id $holder.Id -Force -ErrorAction SilentlyContinue; $holder = $null
    Start-Sleep -Seconds 33
    [void][MT]::SetForegroundWindow($calc); Start-Sleep -Milliseconds 400
    [MT]::CtrlAltUp()
    Check 'liberata, dopo il nuovo tentativo funziona' (Wait-Until { IsOpen } 4)
    Close-Mc
    Check 'l avviso non e comparso una seconda volta' ([MT]::MessageBoxEosOS() -eq [IntPtr]::Zero)
  }
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  try { [void](Inv 'mc_close') } catch {}
  if ($holder) { Stop-Process -Id $holder.Id -Force -ErrorAction SilentlyContinue }
  Remove-Item $holderFile -ErrorAction SilentlyContinue
  if ($calc -ne [IntPtr]::Zero -and [MT]::IsWindow($calc)) { [void][MT]::PostMessage($calc, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Start-Sleep 1
  Cleanup-Eos
  if ($mcBackup) { [System.IO.File]::WriteAllText($mc, $mcBackup, (New-Object System.Text.UTF8Encoding($false))) } else { Remove-Item $mc -ErrorAction SilentlyContinue }
  if ($badBackup) { [System.IO.File]::WriteAllText($mcBad, $badBackup, (New-Object System.Text.UTF8Encoding($false))) } else { Remove-Item $mcBad -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
