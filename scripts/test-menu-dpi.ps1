# Verifica del menu globale a scale diverse: la barra mostra i menu e la tendina si apre SOTTO il titolo cliccato,
# dentro il monitor, su ENTRAMBE le barre (una per monitor) mentre la scala del monitor primario cambia davvero.
# Si cambia la scala del primario (125, 150, 175 se supportate) e si ripristina SEMPRE alla fine (blocco finally +
# un processo di sicurezza che la rimette dopo 10 minuti). Una finestra di prova NUOVA con un menu classico; nessun clic
# oltre alla barra di EosOS (via CDP), nessun tasto.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte.
#
# Uso: powershell -File scripts\test-menu-dpi.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MZ {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function JsB([int]$i, [string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Focus([IntPtr]$h) {
  for ($try = 0; $try -lt 4; $try++) {
    [void][MZ]::SetForegroundWindow($h)
    if (Wait-Until { [MZ]::GetForegroundWindow() -eq $h } 1) { return }
    [MZ]::keybd_event(0x12, 0, 0, [UIntPtr]::Zero); [MZ]::keybd_event(0x12, 0, 2, [UIntPtr]::Zero)
    Start-Sleep -Milliseconds 300
  }
}
function Panel-Open { [MZ]::Find('EosOS Pannello') -ne 0 }
function Close-Panel { [void](JsB 0 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null }

$formA = Join-Path $env:TEMP 'eosos-menudpi-a.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-DPI-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
foreach ($n in 'File', 'Modifica', 'Visualizza', 'Aiuto') { $i = New-Object System.Windows.Forms.MenuItem $n; foreach ($v in 'Prima voce', 'Seconda voce', 'Terza voce') { [void]$i.MenuItems.Add($v) }; [void]$m.MenuItems.Add($i) }
$f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8

# Prova a un livello di scala: per ogni barra (una per monitor) apre il menu "Modifica" e controlla dove sta la tendina.
function Test-Scale([string]$etichetta) {
  Write-Host "=== $etichetta ==="
  foreach ($i in 0, 1) {
    $dpr = [double](JsB $i 'bar' 'window.devicePixelRatio')
    $sx = [int](JsB $i 'bar' 'window.screenX'); $sy = [int](JsB $i 'bar' 'window.screenY')
    $n = [int](JsB $i 'bar' "document.querySelectorAll('.app-menu').length")
    Check "barra $i (scala $dpr): mostra i 4 menu (nome e Finestra nascosti di base)" ($n -eq 4) "menu $n"
    $r = JsB $i 'bar' "(function(){var r=document.querySelectorAll('.app-menu')[2].getBoundingClientRect();return [r.left,r.top,r.width,r.height]})()"
    $titleL = $sx + [int]([math]::Round($r[0] * $dpr)); $titleT = $sy + [int]([math]::Round($r[1] * $dpr))
    $titleR = $titleL + [int]([math]::Round($r[2] * $dpr)); $titleB = $titleT + [int]([math]::Round($r[3] * $dpr))
    Close-Panel
    [void](JsB $i 'bar' "document.querySelectorAll('.app-menu')[2].click()")
    Check "barra $i : la tendina si apre" (Wait-Until { (Panel-Open) -and ([int](JsB 0 'panel' "document.querySelectorAll('.menu-item').length") -gt 0) } 6)
    if (Panel-Open) {
      $items = [int](JsB 0 'panel' "document.querySelectorAll('.menu-item').length")
      Check "barra $i : Modifica ha 3 voci" ($items -eq 3) "voci $items"
      $pr = [MZ]::Rect([MZ]::Find('EosOS Pannello'))
      $mon = [System.Windows.Forms.Screen]::AllScreens | Where-Object { $_.Bounds.Left -le $titleL -and $titleL -lt $_.Bounds.Right } | Select-Object -First 1
      Check "barra $i : la tendina sta sotto il titolo (non lo copre)" ($pr[1] -ge $titleB) "tendina in alto a $($pr[1]), titolo finisce a $titleB"
      Check "barra $i : la tendina e vicina al titolo, non lontana piu di 40 px fisici" (($pr[1] - $titleB) -le 40) "distanza $($pr[1] - $titleB)"
      $cx = ($titleL + $titleR) / 2; $pcx = ($pr[0] + $pr[2]) / 2
      Check "barra $i : centrata sotto il titolo (o spostata solo per restare nel monitor)" (([math]::Abs($pcx - $cx) -le 3) -or ($pr[0] -le $mon.Bounds.Left + 12) -or ($pr[2] -ge $mon.Bounds.Right - 12)) "centro tendina $pcx, titolo $cx"
      Check "barra $i : dentro il monitor" (($pr[0] -ge $mon.Bounds.Left) -and ($pr[2] -le $mon.Bounds.Right) -and ($pr[3] -le $mon.Bounds.Bottom)) "tendina $($pr -join ',') monitor $($mon.Bounds)"
      Check "barra $i : larghezza della tendina = 300 px CSS a questa scala" ([math]::Abs(($pr[2] - $pr[0]) - [math]::Round(300 * $dpr)) -le 2) ("larghezza " + ($pr[2] - $pr[0]))
    }
    Close-Panel
  }
}

$orig = Get-PrimaryScale
$supported = @(Get-PrimaryScaleRange)
Write-Host "Scala originale: $orig%. Supportate: $($supported -join '% ')%"
$dead = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $orig, '-After', 600 -PassThru -WindowStyle Hidden
try {
  Cleanup-Eos; Start-Sleep 1
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formA | Out-Null
  $a = 0; Wait-Until { $script:a = [MZ]::Find('EosOS-DPI-A'); $script:a -ne 0 } 20 | Out-Null
  Check 'finestra di prova creata' ($script:a -ne 0)
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Focus ([IntPtr]$script:a)
  Check 'la riga della finestra di prova e letta' (Wait-Until { $r = JsB 0 'launcher' "window.__TAURI_INTERNALS__.invoke('menubar_current')"; ($r.read -eq $true) -and (@($r.menus).Count -eq 4) } 8)
  Test-Scale "scala di partenza $orig%"
  foreach ($s in 125, 150, 175) {
    if ($supported -notcontains $s -or $s -eq $orig) { continue }
    Write-Host "Cambio la scala del PRIMARIO a $s% ..."
    $got = Set-PrimaryScale $s
    Start-Sleep 5
    if ($got -ne $s) { Write-Host "  (scala ottenuta $got)"; }
    Focus ([IntPtr]$script:a); Start-Sleep -Milliseconds 1500
    Test-Scale "scala $s%"
  }
}
finally {
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menudpi-*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Remove-Item $formA -ErrorAction SilentlyContinue
  if ((Get-PrimaryScale) -ne $orig) { [void](Set-PrimaryScale $orig); Start-Sleep 3 }
  if ((Get-PrimaryScale) -eq $orig) { Stop-Process -Id $dead.Id -Force -ErrorAction SilentlyContinue }
  Write-Host "Scala dopo il ripristino: $(Get-PrimaryScale)%"
}
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host 'Tutto OK' }
