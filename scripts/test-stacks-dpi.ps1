# Pile nel Dock a scale dello schermo diverse: cambiando davvero la scala del monitor primario (125, 150, 175 se
# supportate) il pannello di una pila (cartella MIA in %TEMP%) si apre SOPRA la sua icona, dentro il monitor, largo 300 px
# CSS per la scala, con la griglia dentro i bordi. La scala viene ripristinata SEMPRE (finally + processo di sicurezza dopo
# 10 minuti). mission.json salvato e ripristinato. Nessun clic su icone vere del Dock.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte.
# Uso: powershell -File scripts\test-stacks-dpi.ps1 [-Exe percorso\eosos.exe]
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
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Panel-Open { [SD]::Find('EosOS Pannello') -ne 0 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }

function Test-Scale([string]$etichetta) {
  Write-Host "=== $etichetta ==="
  $dpr = [double](Js 'dock' 'window.devicePixelRatio')
  Check "il Dock ha una sola pila (la mia)" (Wait-Until { (Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-stack').length") -eq 1 } 8)
  $sx = [int](Js 'dock' 'window.screenX'); $sy = [int](Js 'dock' 'window.screenY')
  $r = Js 'dock' "(function(){var r=document.querySelector('.dock:not(.dock-ghost) .dock-stack').getBoundingClientRect();return [r.left,r.top,r.width,r.height]})()"
  # window.screenX/Y sono in px CSS (non fisici): si moltiplica tutto per la scala.
  $iconTop = [int]([math]::Round(($sy + $r[1]) * $dpr)); $iconCx = [int]([math]::Round(($sx + $r[0] + $r[2] / 2) * $dpr))
  Close-Panel
  [void](Js 'dock' "document.querySelector('.dock:not(.dock-ghost) .dock-stack').click()")
  Check "scala ${dpr}: il pannello si apre con i file" (Wait-Until { (Panel-Open) -and ((Js 'panel' "document.querySelectorAll('.stack-file').length") -ge 5) } 8)
  if (-not (Panel-Open)) { return }
  Start-Sleep -Milliseconds 900
  $pr = [SD]::Rect([SD]::Find('EosOS Pannello'))
  $mon = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  Check "sopra l'icona" ($pr[3] -le ($iconTop + 4)) "fondo pannello $($pr[3]), cima icona $iconTop"
  Check "dentro il monitor" (($pr[0] -ge $mon.Left) -and ($pr[2] -le $mon.Right) -and ($pr[1] -ge $mon.Top)) "pannello $($pr -join ',')"
  Check "larghezza 300 px CSS per questa scala" ([math]::Abs(($pr[2] - $pr[0]) - [math]::Round(300 * $dpr)) -le 2) ("larghezza " + ($pr[2] - $pr[0]))
  Check "centrato sull'icona (o contro il bordo)" (([math]::Abs((($pr[0] + $pr[2]) / 2) - $iconCx) -le 3) -or ($pr[2] -ge $mon.Right - [math]::Ceiling(9 * $dpr))) "centro pannello $((($pr[0] + $pr[2]) / 2)), icona $iconCx"
  Check "la griglia non esce dai bordi" ((Js 'panel' "(function(){var g=document.querySelector('.stack-grid');return g.scrollWidth<=g.clientWidth+1&&document.documentElement.scrollWidth<=window.innerWidth+1})()") -eq $true)
  Check "tutto il contenuto e dentro la finestra" ((Js 'panel' "(function(){var b=document.querySelector('.stack').getBoundingClientRect();return b.bottom<=window.innerHeight+1&&b.top>=-1})()") -eq $true)
  Close-Panel
}

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$root = Join-Path $env:TEMP ("eosos-stacks-dpi-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
$dir = Join-Path $root 'ProvaPila'
New-Item -ItemType Directory -Force $dir | Out-Null
1..12 | ForEach-Object { Set-Content (Join-Path $dir ("file{0:D2}-con-un-nome-abbastanza-lungo.txt" -f $_)) 'x' }
$orig = Get-PrimaryScale
$supported = @(Get-PrimaryScaleRange)
Write-Host "Scala originale: $orig%. Supportate: $($supported -join '% ')%"
$dead = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $orig, '-After', 600 -PassThru -WindowStyle Hidden
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $esc = $dir.Replace('\', '\\')
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.stacks',value:['$esc']})")
  Test-Scale "scala di partenza $orig%"
  foreach ($s in 125, 150, 175) {
    if ($supported -notcontains $s -or $s -eq $orig) { continue }
    Write-Host "Cambio la scala del PRIMARIO a $s% ..."
    $got = Set-PrimaryScale $s
    Start-Sleep 6
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
  if ($root -like "$env:TEMP\eosos-stacks-dpi-*") { Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue }
  Write-Host "Scala dopo il ripristino: $(Get-PrimaryScale)%"
}
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host 'Tutto OK' }
