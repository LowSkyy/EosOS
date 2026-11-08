# Il pannello sotto l'orologio (notifiche in alto, calendario sotto) a scale dello schermo diverse: cambiando davvero la
# scala del monitor primario (125, 150, 175 se supportate), su ENTRAMBE le barre (una per monitor) il pannello si apre
# SOTTO l'orologio cliccato, dentro il monitor, con la larghezza prevista per la scala, e il suo contenuto non esce
# dai bordi. La scala viene ripristinata SEMPRE (blocco finally + un processo di sicurezza dopo 10 minuti). La lettura
# delle notifiche resta spenta (mission.json salvato e ripristinato): nessuna notifica viene letta.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte.
#
# Uso: powershell -File scripts\test-calendar-dpi.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class CD {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function JsB([int]$i, [string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Panel-Open { [CD]::Find('EosOS Pannello') -ne 0 }
function Close-Panel { [void](JsB 0 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }

function Test-Scale([string]$etichetta) {
  Write-Host "=== $etichetta ==="
  foreach ($i in 0, 1) {
    $dpr = [double](JsB $i 'bar' 'window.devicePixelRatio')
    $sx = [int](JsB $i 'bar' 'window.screenX'); $sy = [int](JsB $i 'bar' 'window.screenY')
    $r = JsB $i 'bar' "(function(){var r=document.querySelector('.clock').getBoundingClientRect();return [r.left,r.top,r.width,r.height]})()"
    $cl = $sx + [int]([math]::Round($r[0] * $dpr)); $ct = $sy + [int]([math]::Round($r[1] * $dpr))
    $cr = $cl + [int]([math]::Round($r[2] * $dpr)); $cb = $ct + [int]([math]::Round($r[3] * $dpr))
    Close-Panel
    [void](JsB $i 'bar' "document.querySelector('.clock').click()")
    Check "barra $i (scala $dpr): il pannello si apre" (Wait-Until { (Panel-Open) -and ((JsB 0 'panel' "document.querySelectorAll('.cal-day').length") -eq 42) } 6)
    if (-not (Panel-Open)) { continue }
    Start-Sleep -Milliseconds 500
    $pr = [CD]::Rect([CD]::Find('EosOS Pannello'))
    $mon = [System.Windows.Forms.Screen]::AllScreens | Where-Object { $_.Bounds.Left -le $cl -and $cl -lt $_.Bounds.Right } | Select-Object -First 1
    Check "barra $i : sotto l'orologio, vicino" (($pr[1] -ge $cb) -and (($pr[1] - $cb) -le 40)) "pannello in alto a $($pr[1]), orologio finisce a $cb"
    Check "barra $i : dentro il monitor" (($pr[0] -ge $mon.Bounds.Left) -and ($pr[2] -le $mon.Bounds.Right) -and ($pr[3] -le $mon.Bounds.Bottom)) "pannello $($pr -join ',')"
    Check "barra $i : larghezza 300 px CSS per questa scala" ([math]::Abs(($pr[2] - $pr[0]) - [math]::Round(300 * $dpr)) -le 2) ("larghezza " + ($pr[2] - $pr[0]))
    Check "barra $i : la tabella del calendario non esce dai bordi" ((JsB 0 'panel' "(function(){var t=document.querySelector('.cal-grid');return t.scrollWidth<=t.clientWidth+1&&document.documentElement.scrollWidth<=window.innerWidth+1})()") -eq $true)
    Check "barra $i : tutto il calendario e dentro la finestra (nessun taglio in basso)" ((JsB 0 'panel' "(function(){var b=document.querySelector('.cal-grid').getBoundingClientRect();return b.bottom<=window.innerHeight+1})()") -eq $true)
    Close-Panel
  }
}

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$orig = Get-PrimaryScale
$supported = @(Get-PrimaryScaleRange)
Write-Host "Scala originale: $orig%. Supportate: $($supported -join '% ')%"
$dead = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $orig, '-After', 600 -PassThru -WindowStyle Hidden
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Test-Scale "scala di partenza $orig%"
  foreach ($s in 125, 150, 175) {
    if ($supported -notcontains $s -or $s -eq $orig) { continue }
    Write-Host "Cambio la scala del PRIMARIO a $s% ..."
    $got = Set-PrimaryScale $s
    Start-Sleep 5
    if ($got -ne $s) { Write-Host "  (scala ottenuta $got)" }
    Test-Scale "scala $s%"
  }
}
finally {
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
  if ((Get-PrimaryScale) -ne $orig) { [void](Set-PrimaryScale $orig); Start-Sleep 3 }
  if ((Get-PrimaryScale) -eq $orig) { Stop-Process -Id $dead.Id -Force -ErrorAction SilentlyContinue }
  Write-Host "Scala dopo il ripristino: $(Get-PrimaryScale)%"
}
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host 'Tutto OK' }
