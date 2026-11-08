# Verifica della finestra delle impostazioni a scale diverse: cambiando davvero la scala del monitor primario
# (125, 150, 175 se supportate) la finestra si apre al centro del monitor dove sta il cursore, ha la misura prevista
# per quella scala (780 x 560 px CSS, ridotta se il monitor e piccolo), resta tutta dentro il monitor e il suo
# contenuto non esce dai bordi. La scala viene ripristinata SEMPRE (blocco finally + un processo di sicurezza dopo
# 10 minuti). I quattro file delle impostazioni sono salvati e ripristinati. Il cursore si porta al centro del
# monitor primario (e poi si rimette dov'era).
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte.
#
# Uso: powershell -File scripts\test-settings-dpi.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class SD {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern bool GetCursorPos(out P p);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }

function Test-Scale([string]$etichetta, [double]$scale) {
  Write-Host "=== $etichetta ==="
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_close')"); Start-Sleep -Milliseconds 500
  $prim = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  [void][SD]::SetCursorPos($prim.Left + [int]($prim.Width / 2), $prim.Top + [int]($prim.Height / 2))
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'dock'})")
  Check 'la finestra si apre' (Wait-Until { [SD]::Find('EosOS Impostazioni') -ne 0 } 6)
  Start-Sleep -Milliseconds 1200
  $h = [SD]::Find('EosOS Impostazioni'); if ($h -eq 0) { return }
  $r = [SD]::Rect($h)
  $w = $r[2] - $r[0]; $hh = $r[3] - $r[1]
  $mw = [math]::Min(780 * $scale, $prim.Width - 2 * 24 * $scale); $mh = [math]::Min(560 * $scale, $prim.Height - 2 * 24 * $scale)
  Check 'misura prevista per questa scala' (([math]::Abs($w - [math]::Round($mw)) -le 2) -and ([math]::Abs($hh - [math]::Round($mh)) -le 2)) "${w}x${hh}, attesa $([math]::Round($mw))x$([math]::Round($mh))"
  Check 'al centro del monitor primario' (([math]::Abs((($r[0] + $r[2]) / 2) - ($prim.Left + $prim.Width / 2)) -le 3) -and ([math]::Abs((($r[1] + $r[3]) / 2) - ($prim.Top + $prim.Height / 2)) -le 3)) ("finestra " + ($r -join ','))
  Check 'tutta dentro il monitor' (($r[0] -ge $prim.Left) -and ($r[1] -ge $prim.Top) -and ($r[2] -le $prim.Right) -and ($r[3] -le $prim.Bottom))
  $dpr = [double](Js 'settings' 'window.devicePixelRatio')
  Check 'la pagina conosce la scala' ([math]::Abs($dpr - $scale) -le 0.02) "devicePixelRatio $dpr"
  Check 'il contenuto non esce di lato (nessuna barra di scorrimento orizzontale)' ((Js 'settings' 'document.documentElement.scrollWidth <= window.innerWidth + 1') -eq $true)
  Check 'barra del titolo, ricerca, sezioni e prima riga sono dentro la finestra' ((Js 'settings' "(function(){var ok=true;['.settings-titlebar','.settings-search','.settings-nav','.settings-row'].forEach(function(s){var e=document.querySelector(s);if(!e){ok=false;return}var b=e.getBoundingClientRect();if(b.left<0||b.top<0||b.right>window.innerWidth+1||b.bottom>window.innerHeight+1)ok=false});return ok})()") -eq $true)
  Check 'il pulsante di chiusura e raggiungibile (dentro la finestra, non coperto)' ((Js 'settings' "(function(){var b=document.querySelector('.settings-close').getBoundingClientRect();var e=document.elementFromPoint(b.left+b.width/2,b.top+b.height/2);return !!e&&e.closest('.settings-close')!==null})()") -eq $true)
}

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
$orig = Get-PrimaryScale
$supported = @(Get-PrimaryScaleRange)
Write-Host "Scala originale: $orig%. Supportate: $($supported -join '% ')%"
$cur0 = New-Object SD+P; [void][SD]::GetCursorPos([ref]$cur0)
$dead = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $orig, '-After', 600 -PassThru -WindowStyle Hidden
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Test-Scale "scala di partenza $orig%" ($orig / 100)
  foreach ($s in 125, 150, 175) {
    if ($supported -notcontains $s -or $s -eq $orig) { continue }
    Write-Host "Cambio la scala del PRIMARIO a $s% ..."
    $got = Set-PrimaryScale $s
    Start-Sleep 5
    if ($got -ne $s) { Write-Host "  (scala ottenuta $got)" }
    Test-Scale "scala $s%" ($got / 100)
  }
}
finally {
  [void][SD]::SetCursorPos($cur0.X, $cur0.Y)
  Cleanup-Eos
  foreach ($f in $files) {
    $p = Join-Path $dir $f
    Remove-Item $p -ErrorAction SilentlyContinue
    if ($null -ne $backup[$f]) { [System.IO.File]::WriteAllBytes($p, $backup[$f]) }
  }
  if ((Get-PrimaryScale) -ne $orig) { [void](Set-PrimaryScale $orig); Start-Sleep 3 }
  if ((Get-PrimaryScale) -eq $orig) { Stop-Process -Id $dead.Id -Force -ErrorAction SilentlyContinue }
  Write-Host "Scala dopo il ripristino: $(Get-PrimaryScale)%"
}
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host 'Tutto OK' }
