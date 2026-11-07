# Prova il Dock a scale (DPI) diverse del monitor primario, CAMBIANDO davvero la scala dello
# schermo e ripristinandola alla fine (blocco finally + processo di sicurezza dpi-deadman.ps1).
# Non fa clic: sposta soltanto il cursore.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte (per qualche secondo).
#
# Uso: powershell -File scripts\test-dock-scales.ps1 [-Exe percorso\eosos.exe] [-Scales 125,150,175]
#
# Per ogni scala (con EosOS gia' acceso, quindi a cambio di scala "dal vivo") e poi una volta con
# avvio a freddo alla scala piu' alta controlla:
#   - il rapporto di pixel della pagina (devicePixelRatio) = scala
#   - altezza della finestra del Dock = ceil(168 * scala); centrata; appoggiata sopra la taskbar
#   - barra in alto: area di lavoro che parte da round(28 * scala)
#   - riserva di spazio: fondo dell'area di lavoro = bordo della taskbar - round(92 * scala)
#   - zona cliccabile: a riposo solo la capsula blocca i clic, con il cursore sopra anche la zona sopra
#   - ingrandimento: l'icona sotto il cursore cresce (coordinate del cursore ben mappate sui px CSS)
#   - sfondo sfocato: la capsula coincide con lo schermo dietro di lei (finestra a strisce)
param([string]$Exe, [int[]]$Scales = @(125, 150, 175))
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Remove-Item env:EOSOS_DOCK_RESERVE -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class TS {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref R r, uint c);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  public static R Work() { R r = new R(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
  public static IntPtr FindTitle(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  public static bool IsDockAt(int x, int y, IntPtr dock) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2) == dock; }
}
'@
$fail = 0; $rows = @()
function Check($l, $c) { if ($c) { Write-Host "  [OK]    $l" } else { Write-Host "  [FALLITO] $l" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; $p = Start-Process $exe -PassThru; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 9; $p }
function Wait-Dock { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 20) { $h = [TS]::FindTitle('EosOS Dock'); if ($h -ne [IntPtr]::Zero) { return $h }; Start-Sleep -Milliseconds 300 }; [IntPtr]::Zero }
function Cdp([string]$expr) { (& node $cdp eval $expr) | ConvertFrom-Json }
function Rect($h) { $r = New-Object TS+R; [void][TS]::GetWindowRect($h, [ref]$r); $r }
$stripes = $null
function Stop-Stripes { if ($script:stripes) { Stop-Process -Id $script:stripes.Id -Force -ErrorAction SilentlyContinue; $script:stripes = $null } }

function Run-Checks([int]$pct, [string]$label) {
  $k = $pct / 100.0
  Write-Host "`n=== $label : scala $pct% ===" -ForegroundColor Cyan
  $dock = Wait-Dock
  Check "finestra del Dock presente e visibile" ($dock -ne [IntPtr]::Zero)
  if ($dock -eq [IntPtr]::Zero) { $script:rows += [pscustomobject]@{ Scala = "$pct%"; Prova = $label; Esito = 'FALLITO (Dock assente)' }; return }
  Start-Sleep 2   # il ciclo di controllo rilegge monitor e scala ogni secondo
  $failBefore = $script:fail
  $dr = Rect $dock; $w = [TS]::Work(); $tray = New-Object TS+R; [void][TS]::GetWindowRect([TS]::FindWindow('Shell_TrayWnd', $null), [ref]$tray)
  $screenW = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds.Width
  $dpr = [double](Cdp 'devicePixelRatio')
  Write-Host ("  Dock {0}x{1} in ({2},{3}); area di lavoro {4}..{5}; taskbar da {6}; dpr {7}" -f ($dr.Rt - $dr.L), ($dr.B - $dr.T), $dr.L, $dr.T, $w.T, $w.B, $tray.T, $dpr)
  Check "rapporto di pixel della pagina = scala ($dpr)" ([Math]::Abs($dpr - $k) -lt 0.01)
  Check "altezza del Dock = ceil(168 x scala) = $([Math]::Ceiling(168 * $k - 1e-9))" ([Math]::Abs(($dr.B - $dr.T) - [Math]::Ceiling(168 * $k - 1e-9)) -le 1)
  Check "Dock centrato orizzontalmente" ([Math]::Abs((($dr.L + $dr.Rt) / 2) - ($screenW / 2)) -le 2)
  Check "Dock appoggiato sopra la taskbar a round(8 x scala) = $([Math]::Round(8 * $k))" ([Math]::Abs($dr.B - ($tray.T - [Math]::Round(8 * $k))) -le 2)
  Check "barra in alto: area di lavoro parte da round(28 x scala) = $([Math]::Round(28 * $k))" ([Math]::Abs($w.T - [Math]::Round(28 * $k)) -le 1)
  Check "riserva di spazio: fondo dell'area di lavoro = taskbar - round(92 x scala) = $($tray.T - [Math]::Round(92 * $k))" ([Math]::Abs($w.B - ($tray.T - [Math]::Round(92 * $k))) -le 2)

  # Zona cliccabile (nessun clic: si chiede solo quale finestra c'e' sotto il cursore)
  $cx = [int](($dr.L + $dr.Rt) / 2); $above = [int]($dr.T + 20 * $k); $inside = [int]($dr.B - 30 * $k)
  [void][TS]::SetCursorPos(5, 300); Start-Sleep 0.8
  Check "a riposo la zona sopra la capsula non blocca i clic" (-not [TS]::IsDockAt($cx, $above, $dock))
  Check "a riposo la capsula blocca i clic" ([TS]::IsDockAt($cx, $inside, $dock))
  [void][TS]::SetCursorPos($cx, $inside); Start-Sleep 0.9
  Check "col cursore sul Dock la zona estesa blocca i clic" ([TS]::IsDockAt($cx, $above, $dock))

  # Ingrandimento: l'icona piu' ingrandita dev'essere quella piu' vicina al cursore. Se le coordinate
  # del cursore non fossero mappate bene sui px CSS (scala), la piu' grande sarebbe un'altra.
  [void][TS]::SetCursorPos(5, 300); Start-Sleep 1.0
  $items = Cdp "JSON.stringify([...document.querySelectorAll('.dock:not(.dock-ghost) .dock-item')].map(e=>{const b=e.getBoundingClientRect();return [e.title,b.left+b.width/2,b.top+b.height/2,b.width]}))" | ConvertFrom-Json
  $target = $items | Where-Object { $_[0] -eq 'Visual Studio Code' } | Select-Object -First 1
  if (-not $target) { $target = $items | Select-Object -Index ([int]($items.Count / 2)) }
  $px = [int]($dr.L + $target[1] * $k); $py = [int]($dr.T + $target[2] * $k)
  [void][TS]::SetCursorPos($px, $py); Start-Sleep 1.0
  $now = Cdp "JSON.stringify([...document.querySelectorAll('.dock:not(.dock-ghost) .dock-item')].map(e=>{const b=e.getBoundingClientRect();return [e.title,b.left+b.width/2,b.width]}))" | ConvertFrom-Json
  $cursorCss = ($px - $dr.L) / $k
  $nearest = ($now | Sort-Object { [Math]::Abs($_[1] - $cursorCss) } | Select-Object -First 1)
  $biggest = ($now | Sort-Object { -$_[2] } | Select-Object -First 1)
  Check ("l'icona piu' ingrandita ('{0}', larga {1:N0} px CSS, ~95 attesi) e' la piu' vicina al cursore ('{2}')" -f $biggest[0], $biggest[2], $nearest[0]) ($biggest[0] -eq $nearest[0] -and $biggest[2] -gt 85)
  [void][TS]::SetCursorPos(5, 300); Start-Sleep 0.5

  # Sfondo sfocato: la capsula (senza sfocatura ne' tinta) deve coincidere con lo schermo dietro di lei
  $script:stripes = Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'test-stripes-window.ps1') -PassThru -WindowStyle Hidden
  Start-Sleep 4
  $blurOn = $false; $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 8) { if ((Cdp "document.documentElement.getAttribute('data-blur')") -eq 'on') { $blurOn = $true; break }; Start-Sleep -Milliseconds 300 }
  Check "sfocatura attiva (arrivano i fotogrammi dello sfondo)" $blurOn
  if ($blurOn) {
    Start-Sleep 1.5
    $png = Join-Path $env:TEMP "dock-scale-$pct.png"
    $inject = "(()=>{const s=document.createElement('style');s.id='t';s.textContent='.dock-blur{filter:none!important}.dock::before{display:none!important}.dock-item,.dock-separator{visibility:hidden!important}';document.head.appendChild(s)})()"
    $inner = [double](& node $cdp shot $png $inject)
    $cap = Cdp "JSON.stringify((()=>{const b=document.querySelector('.dock:not(.dock-ghost)').getBoundingClientRect();return [b.left,b.top,b.width,b.height]})())" | ConvertFrom-Json
    [void](& node $cdp eval "document.getElementById('t')?.remove()")
    $img = [Drawing.Bitmap]::FromFile($png); $kk = $img.Width / $inner
    $sw = [int]($cap[2] * $k); $sh = [int]($cap[3] * $k)
    $scr = New-Object Drawing.Bitmap $sw, $sh
    $g = [Drawing.Graphics]::FromImage($scr); $g.CopyFromScreen([int]($dr.L + $cap[0] * $k), [int]($dr.T + $cap[1] * $k), 0, 0, $scr.Size); $g.Dispose()
    $ok = 0; $tot = 0
    try {
      for ($y = 14; $y -lt $cap[3] - 14; $y += 3) { for ($x = 40; $x -lt $cap[2] - 40; $x += 3) {
        $sx = [Math]::Min([int]($x * $k), $scr.Width - 1); $sy = [Math]::Min([int]($y * $k), $scr.Height - 1)
        $ix = [Math]::Min([int](($cap[0] + $x) * $kk), $img.Width - 1); $iy = [Math]::Min([int](($cap[1] + $y) * $kk), $img.Height - 1)
        $a = $scr.GetPixel($sx, $sy); $b = $img.GetPixel($ix, $iy)
        $tot++; if ([Math]::Abs(($a.R + $a.G + $a.B) / 3 - ($b.R + $b.G + $b.B) / 3) -lt 60) { $ok++ } } }
    } catch { Write-Host "  (confronto interrotto: $($_.Exception.Message))" -ForegroundColor Yellow }
    $img.Dispose(); $scr.Dispose()
    $pctOk = [Math]::Round(100 * $ok / [Math]::Max(1, $tot), 1)
    Check "sfondo sfocato allineato allo schermo dietro la capsula ($pctOk% dei pixel concordi, soglia 90%)" ($pctOk -ge 90)
  }
  Stop-Stripes
  $esito = if ($script:fail -eq $failBefore) { 'OK' } else { 'FALLITO' }
  $script:rows += [pscustomobject]@{ Scala = "$pct%"; Prova = $label; Esito = $esito }
}

# ---------------------------------------------------------------------------------------------
$orig = Get-PrimaryScale
$supported = @(Get-PrimaryScaleRange)
Write-Host "Scala originale: $orig%. Scale supportate dal monitor: $($supported -join '% ')%"
$todo = @($Scales | Where-Object { $_ -ne $orig })
$skipped = @($todo | Where-Object { $supported -notcontains $_ })
if ($skipped) { Write-Host "Non supportate da questo monitor (saltate): $($skipped -join '% ')%" -ForegroundColor Yellow }
$todo = @($todo | Where-Object { $supported -contains $_ })
$dead = Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $orig, '-After', 600 -PassThru -WindowStyle Hidden
Write-Host "Rete di sicurezza attiva (pid $($dead.Id)): fra 10 minuti ripristina il $orig% se serve."
try {
  Cleanup-Eos; Start-Sleep 1
  $p = Start-Eos
  Run-Checks $orig 'dal vivo (scala di partenza)'
  foreach ($s in $todo) {
    Write-Host "`nCambio la scala dello schermo a $s% ..." -ForegroundColor Yellow
    $got = Set-PrimaryScale $s
    if ($got -ne $s) { Write-Host "  il sistema ha dato $got% invece di $s%" -ForegroundColor Red; $fail++; continue }
    Start-Sleep 6
    Run-Checks $s 'cambio di scala con EosOS acceso'
  }
  if ($todo.Count -gt 0) {
    Cleanup-Eos; Start-Sleep 3
    $p = Start-Eos
    Run-Checks (Get-PrimaryScale) 'avvio a freddo'
  }
}
finally {
  Stop-Stripes
  Cleanup-Eos
  Write-Host "`nRipristino la scala originale ($orig%) ..." -ForegroundColor Yellow
  try { $back = Set-PrimaryScale $orig; Write-Host "Scala ora: $back%" } catch { Write-Host "RIPRISTINO FALLITO: $_ (il processo di sicurezza riprovera')" -ForegroundColor Red; $fail++ }
  if ((Get-PrimaryScale) -eq $orig) { Stop-Process -Id $dead.Id -Force -ErrorAction SilentlyContinue }
  Start-Sleep 3
  $wf = [TS]::Work(); Write-Host "Area di lavoro finale: $($wf.T)..$($wf.B)"
}
Write-Host "`nRiepilogo:"; $rows | Format-Table -AutoSize | Out-String | Write-Host
if ($fail -gt 0) { Write-Host "FALLITI: $fail" -ForegroundColor Red; exit 1 } else { Write-Host "Tutto OK" }
