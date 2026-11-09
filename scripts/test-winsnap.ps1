# Gestione delle finestre (snap): su finestre di prova MIE (un modulo ridimensionabile e uno a misura fissa, mai finestre dell'utente) si
# eseguono le azioni (metà, quarti, terzi, massimizza, centra, ripristina, altro monitor) con il comando di prova `winsnap_fire` e si
# controlla che la parte VISIBILE della finestra (cornice DWM, senza i bordi invisibili) finisca dove deve nell'area di lavoro del
# monitor. Poi l'impostazione: spente di base, le combinazioni si registrano solo scegliendo un modificatore, una scelta inventata e
# rifiutata. mission.json e' salvato e ripristinato. Non si simula nessun tasto.
#
# Uso: powershell -File scripts\test-winsnap.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);' -Name Dpi -Namespace SnapTest
[void][SnapTest.Dpi]::SetProcessDpiAwarenessContext([IntPtr](-4))
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class SN {
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out R r, int n);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int hh, uint f);
 [DllImport("user32.dll")] static extern bool IsWindow(IntPtr h);
 public static int[] Frame(long h) { R r; if (DwmGetWindowAttribute(new IntPtr(h), 9, out r, 16) != 0) GetWindowRect(new IntPtr(h), out r); return new int[]{ r.L, r.T, r.Rt, r.B }; }
 public static bool Alive(long h) { return IsWindow(new IntPtr(h)); }
 public static void Move(long h, int x, int y, int w, int hh) { SetWindowPos(new IntPtr(h), IntPtr.Zero, x, y, w, hh, 0x14); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Fire([string]$name, [long]$h) { Js "window.__TAURI_INTERNALS__.invoke('winsnap_fire',{name:'$name',hwnd:$h})" }
function Set-S([string]$key, [string]$jsValue) { Js "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$key',value:$jsValue})" }
function Val([string]$key) { (@(Js "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq $key }).value }
function Registered { [int](Js "window.__TAURI_INTERNALS__.invoke('winsnap_registered')") }
# Il confine i-esimo di len pixel divisi in parts (come nel codice: il resto va all'ultima parte).
function Cut([int]$start, [int]$len, [int]$i, [int]$parts) { $start + [int][math]::Floor($len * $i / $parts) }
function Near($a, $b, [int]$tol = 2) { $ok = $true; for ($i = 0; $i -lt 4; $i++) { if ([math]::Abs($a[$i] - $b[$i]) -gt $tol) { $ok = $false } }; $ok }
function Rs($r) { "[" + ($r -join ',') + "]" }
function Open-Win([string]$title, [switch]$Fixed, [int]$x = 400, [int]$y = 300) {
  $hf = Join-Path $env:TEMP "snap-$([guid]::NewGuid().ToString('N')).txt"
  $a = @('-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'snap-window.ps1'), '-Title', $title, '-HandleFile', $hf, '-X', $x, '-Y', $y)
  if ($Fixed) { $a += '-Fixed' }
  $p = Start-Process powershell -ArgumentList $a -PassThru -WindowStyle Hidden
  if (-not (Wait-Until { Test-Path $hf } 15)) { Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue; throw "finestra di prova $title non comparsa" }
  [pscustomobject]@{ Pid = $p.Id; H = [long](Get-Content $hf); File = $hf }
}

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bk = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$wins = @()
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 13

  $screens = @([System.Windows.Forms.Screen]::AllScreens | Sort-Object { $_.Bounds.Left }, { $_.Bounds.Top })
  $prim = [System.Windows.Forms.Screen]::PrimaryScreen
  $pw = $prim.WorkingArea
  Write-Host ("monitor: " + $screens.Count + "; primario area di lavoro " + $pw.Left + "," + $pw.Top + " " + $pw.Width + "x" + $pw.Height)

  Write-Host 'A. impostazione'
  Check 'di base le scorciatoie sono spente' ((Val 'snap.keys') -eq 'off')
  Check 'spente: nessuna combinazione registrata' ((Registered) -eq 0) ("registrate " + (Registered))
  $r = Set-S 'snap.keys' "'ctrl_alt'"
  Check 'si sceglie Ctrl+Alt' (($r.ok -eq $true) -and ((Val 'snap.keys') -eq 'ctrl_alt'))
  Check 'le 16 combinazioni si registrano (o quasi: una puo essere gia di un altra app)' (Wait-Until { (Registered) -ge 12 } 4) ("registrate " + (Registered))
  Check 'una scelta inventata e rifiutata' (((Set-S 'snap.keys' "'tutti'").ok) -eq $false)
  Check 'la scelta di prima resta' ((Val 'snap.keys') -eq 'ctrl_alt')
  [void](Set-S 'snap.keys' "'off'")
  Check 'di nuovo spente: le combinazioni si liberano' (Wait-Until { (Registered) -eq 0 } 4) ("registrate " + (Registered))

  Write-Host 'B. disposizioni sul monitor primario'
  $w = Open-Win 'EosOS-Snap-A' -x ($pw.Left + 300) -y ($pw.Top + 200)
  $wins += $w
  Start-Sleep -Milliseconds 600
  $orig = [SN]::Frame($w.H)
  $L = $pw.Left; $T = $pw.Top; $Wd = $pw.Width; $Hd = $pw.Height; $R = $L + $Wd; $B = $T + $Hd
  $cases = [ordered]@{
    left = @($L, $T, (Cut $L $Wd 1 2), $B); right = @((Cut $L $Wd 1 2), $T, $R, $B)
    top = @($L, $T, $R, (Cut $T $Hd 1 2)); bottom = @($L, (Cut $T $Hd 1 2), $R, $B)
    topleft = @($L, $T, (Cut $L $Wd 1 2), (Cut $T $Hd 1 2)); topright = @((Cut $L $Wd 1 2), $T, $R, (Cut $T $Hd 1 2))
    bottomleft = @($L, (Cut $T $Hd 1 2), (Cut $L $Wd 1 2), $B); bottomright = @((Cut $L $Wd 1 2), (Cut $T $Hd 1 2), $R, $B)
    leftthird = @($L, $T, (Cut $L $Wd 1 3), $B); centerthird = @((Cut $L $Wd 1 3), $T, (Cut $L $Wd 2 3), $B); rightthird = @((Cut $L $Wd 2 3), $T, $R, $B)
    maximize = @($L, $T, $R, $B)
  }
  foreach ($k in $cases.Keys) {
    $ok = Fire $k $w.H
    Start-Sleep -Milliseconds 250
    $got = [SN]::Frame($w.H)
    Check ("$k : eseguita e al posto giusto") (($ok -eq $true) -and (Near $got $cases[$k])) ("atteso " + (Rs $cases[$k]) + " trovato " + (Rs $got))
  }

  Write-Host 'C. centra e ripristina'
  [SN]::Move($w.H, $L + 100, $T + 100, 640, 420); Start-Sleep -Milliseconds 250
  $before = [SN]::Frame($w.H)
  [void](Fire 'center' $w.H); Start-Sleep -Milliseconds 250
  $c = [SN]::Frame($w.H)
  $cw = $before[2] - $before[0]; $ch = $before[3] - $before[1]
  $exp = @(($L + [int][math]::Floor(($Wd - $cw) / 2)), ($T + [int][math]::Floor(($Hd - $ch) / 2)))
  Check 'centra: misura invariata' ((($c[2] - $c[0]) -eq $cw) -and (($c[3] - $c[1]) -eq $ch)) ("misura " + $cw + "x" + $ch)
  Check 'centra: al centro dell area di lavoro' (([math]::Abs($c[0] - $exp[0]) -le 2) -and ([math]::Abs($c[1] - $exp[1]) -le 2)) ("atteso " + ($exp -join ',') + " trovato " + $c[0] + "," + $c[1])
  # Due disposizioni di fila e poi «ripristina»: torna alla posizione di PRIMA della prima.
  [SN]::Move($w.H, $L + 150, $T + 120, 640, 420); Start-Sleep -Milliseconds 250
  $start = [SN]::Frame($w.H)
  [void](Fire 'left' $w.H); Start-Sleep -Milliseconds 200; [void](Fire 'topright' $w.H); Start-Sleep -Milliseconds 200
  $ok = Fire 'restore' $w.H; Start-Sleep -Milliseconds 250
  Check 'ripristina: torna dov era prima della prima disposizione' (($ok -eq $true) -and (Near ([SN]::Frame($w.H)) $start)) ("atteso " + (Rs $start) + " trovato " + (Rs ([SN]::Frame($w.H))))
  Check 'ripristina di nuovo: non c e piu niente da ripristinare' ((Fire 'restore' $w.H) -eq $false)

  Write-Host 'D. finestre non adatte'
  $fx = Open-Win 'EosOS-Snap-Fisso' -Fixed -x ($pw.Left + 200) -y ($pw.Top + 150)
  $wins += $fx
  Start-Sleep -Milliseconds 600
  $fb = [SN]::Frame($fx.H)
  Check 'finestra a misura fissa: rifiutata' ((Fire 'left' $fx.H) -eq $false)
  Check 'e non si e mossa' (Near ([SN]::Frame($fx.H)) $fb 1)
  Check 'handle inesistente: rifiutato' ((Fire 'left' 12345) -eq $false)
  Check 'azione inventata: rifiutata' ((Fire 'boh' $w.H) -eq $false)

  Write-Host 'E. due monitor'
  if ($screens.Count -ge 2) {
    $a = $screens[0].WorkingArea; $b = $screens[1].WorkingArea
    $mine = if ($prim.DeviceName -eq $screens[0].DeviceName) { 0 } else { 1 }
    [SN]::Move($w.H, $pw.Left + 200, $pw.Top + 150, 800, 500); Start-Sleep -Milliseconds 300
    [void](Fire 'nextmonitor' $w.H); Start-Sleep -Milliseconds 400
    $g = [SN]::Frame($w.H); $cx = ($g[0] + $g[2]) / 2
    $dest = if ($mine -eq 0) { $screens[1].WorkingArea } else { $screens[0].WorkingArea }
    Check 'nextmonitor: la finestra e ora sull altro monitor' (($cx -ge $dest.Left) -and ($cx -lt $dest.Right)) ("centro x $cx, destinazione " + $dest.Left + ".." + $dest.Right)
    Check 'nextmonitor: sta tutta dentro il monitor di arrivo' (($g[0] -ge $dest.Left) -and ($g[2] -le $dest.Right) -and ($g[1] -ge $dest.Top) -and ($g[3] -le $dest.Bottom)) (Rs $g)
    [void](Fire 'left' $w.H); Start-Sleep -Milliseconds 300
    $g = [SN]::Frame($w.H)
    $half = @($dest.Left, $dest.Top, (Cut $dest.Left $dest.Width 1 2), $dest.Bottom)
    Check 'metà sinistra sul secondo monitor usa la SUA area di lavoro' (Near $g $half) ("atteso " + (Rs $half) + " trovato " + (Rs $g))
    [void](Fire 'prevmonitor' $w.H); Start-Sleep -Milliseconds 400
    $g = [SN]::Frame($w.H); $cx = ($g[0] + $g[2]) / 2
    Check 'prevmonitor: torna sul monitor di partenza' (($cx -ge $pw.Left) -and ($cx -lt $pw.Right)) ("centro x $cx")
  } else { Write-Host '  (un solo monitor: salto)' }

  Write-Host 'F. finestra massimizzata o ridotta a icona'
  [void](Fire 'maximize' $w.H)
  Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);' -Name Show -Namespace SnapTest -ErrorAction SilentlyContinue
  [void][SnapTest.Show]::ShowWindow([IntPtr]$w.H, 2); Start-Sleep -Milliseconds 400   # ridotta a icona
  $ok = Fire 'right' $w.H; Start-Sleep -Milliseconds 400
  $got = [SN]::Frame($w.H)
  $exp = @((Cut $pw.Left $pw.Width 1 2), $pw.Top, $pw.Right, $pw.Bottom)
  Check 'da ridotta a icona: si riporta e va a metà destra' (($ok -eq $true) -and (Near $got $exp 4)) ("atteso " + (Rs $exp) + " trovato " + (Rs $got))
}
finally {
  foreach ($x in $wins) { Stop-Process -Id $x.Pid -Force -ErrorAction SilentlyContinue; Remove-Item $x.File -ErrorAction SilentlyContinue }
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bk) { [System.IO.File]::WriteAllBytes($mc, $bk) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
