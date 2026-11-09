# Prova il Dock con DUE MONITOR A SCALE DIVERSE, cambiando davvero la scala di entrambi e
# ripristinandole alla fine (blocco finally + un processo di sicurezza per ogni monitor cambiato).
# Non fa clic: sposta soltanto il cursore.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte, sul monitor interessato.
#
# Uso: powershell -File scripts\test-dock-mixed-dpi.ps1 [-Exe percorso\eosos.exe] [-SecondaryScale 150] [-PrimaryScale 125]
#
# Fasi (scale del primario / del secondario):
#   1. dal vivo: il secondario passa a -SecondaryScale, il primario resta com'e'
#   2. dal vivo: il primario passa a -PrimaryScale, il secondario resta a -SecondaryScale
#   3. avvio a freddo con le due scale diverse
# A ogni fase si controlla che il Dock (solo sul primario) segua la scala del PRIMARIO e che la
# barra in alto di ciascun monitor segua la scala del PROPRIO monitor.
param([string]$Exe, [int]$SecondaryScale = 150, [int]$PrimaryScale = 125)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Remove-Item env:EOSOS_DOCK_RESERVE -ErrorAction SilentlyContinue
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class MX {
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
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref R r, uint c);
  public static R Work() { R r = new R(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
  public static IntPtr FindTitle(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  // Rettangoli (L,T,R,B) di tutte le finestre visibili con questo titolo (le barre in alto si chiamano "EosOS").
  public static string[] AllTitled(string title) { var o = new List<string>(); EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) { R r; GetWindowRect(h, out r); o.Add(r.L + "," + r.T + "," + r.Rt + "," + r.B); } return true; }, IntPtr.Zero); return o.ToArray(); }
  public static bool IsDockAt(int x, int y, IntPtr dock) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2) == dock; }
}
'@
$fail = 0; $rows = @()
function Check($l, $c) { if ($c) { Write-Host "  [OK]    $l" } else { Write-Host "  [FALLITO] $l" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; $p = Start-Process $exe -PassThru; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 9; $p }
function Wait-Dock { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 20) { $h = [MX]::FindTitle('EosOS Dock'); if ($h -ne [IntPtr]::Zero) { return $h }; Start-Sleep -Milliseconds 300 }; [IntPtr]::Zero }
function Cdp([string]$expr) { (& node $cdp eval $expr) | ConvertFrom-Json }
function Rect($h) { $r = New-Object MX+R; [void][MX]::GetWindowRect($h, [ref]$r); $r }
$stripes = $null
function Stop-Stripes { if ($script:stripes) { Stop-Process -Id $script:stripes.Id -Force -ErrorAction SilentlyContinue; $script:stripes = $null } }

function Run-Checks([string]$label) {
  $mons = @(Get-Monitors); $prim = $mons | Where-Object Primario | Select-Object -First 1; $sec = $mons | Where-Object { -not $_.Primario } | Select-Object -First 1
  $kp = $prim.Scala / 100.0; $ks = $sec.Scala / 100.0
  Write-Host "`n=== $label : primario $($prim.Scala)%, secondario $($sec.Scala)% ===" -ForegroundColor Cyan
  Check "le due scale sono diverse ($($prim.Scala)% e $($sec.Scala)%)" ($prim.Scala -ne $sec.Scala)
  $dock = Wait-Dock
  Check "finestra del Dock presente e visibile" ($dock -ne [IntPtr]::Zero)
  if ($dock -eq [IntPtr]::Zero) { $script:rows += [pscustomobject]@{ Fase = $label; Primario = "$($prim.Scala)%"; Secondario = "$($sec.Scala)%"; Esito = 'FALLITO (Dock assente)' }; return }
  Start-Sleep 2
  $failBefore = $script:fail
  $dr = Rect $dock; $w = [MX]::Work()
  $tray = New-Object MX+R; [void][MX]::GetWindowRect([MX]::FindWindow('Shell_TrayWnd', $null), [ref]$tray)
  $dpr = [double](Cdp 'devicePixelRatio')
  Write-Host ("  Dock {0}x{1} in ({2},{3}); area di lavoro primario {4}..{5}; taskbar da {6}; dpr {7}" -f ($dr.Rt - $dr.L), ($dr.B - $dr.T), $dr.L, $dr.T, $w.T, $w.B, $tray.T, $dpr)

  # --- il Dock segue il primario
  Check "rapporto di pixel della pagina del Dock = scala del PRIMARIO ($dpr)" ([Math]::Abs($dpr - $kp) -lt 0.01)
  Check "altezza del Dock = ceil(168 x scala primario) = $([Math]::Ceiling(168 * $kp - 1e-9))" ([Math]::Abs(($dr.B - $dr.T) - [Math]::Ceiling(168 * $kp - 1e-9)) -le 1)
  $pcx = ($prim.L + $prim.R) / 2
  Check "Dock centrato sul monitor primario" ([Math]::Abs((($dr.L + $dr.Rt) / 2) - $pcx) -le 2)
  Check "Dock tutto dentro il monitor primario (non sconfina sul secondario)" ($dr.L -ge $prim.L -and $dr.Rt -le $prim.R)
  Check "Dock appoggiato sopra la taskbar del primario a round(8 x scala primario)" ([Math]::Abs($dr.B - ($tray.T - [Math]::Round(8 * $kp))) -le 2)
  Check "riserva di spazio sul primario = round(92 x scala primario)" ([Math]::Abs($w.B - ($tray.T - [Math]::Round(92 * $kp))) -le 2)

  # --- ogni barra in alto segue il proprio monitor
  $bars = @([MX]::AllTitled('EosOS') | ForEach-Object { $f = $_ -split ','; [pscustomobject]@{ L = [int]$f[0]; T = [int]$f[1]; R = [int]$f[2]; B = [int]$f[3] } })
  foreach ($m in @($prim, $sec)) {
    $k = $m.Scala / 100.0
    $bar = $bars | Where-Object { $_.L -eq $m.L -and $_.T -eq $m.T } | Select-Object -First 1
    Check "barra in alto sul $($m.Gdi) presente" ($null -ne $bar)
    if ($bar) {
      Check "barra sul $($m.Gdi): altezza round(28 x $($m.Scala)%) = $([Math]::Round(28 * $k))  (e' $($bar.B - $bar.T))" ([Math]::Abs(($bar.B - $bar.T) - [Math]::Round(28 * $k)) -le 1)
      Check "barra sul $($m.Gdi): larga quanto il monitor ($($m.R - $m.L))" (($bar.R - $bar.L) -eq ($m.R - $m.L))
    }
    Check "area di lavoro del $($m.Gdi) parte da round(28 x scala) = $([Math]::Round(28 * $k)) (e' $($m.WorkT - $m.T))" ([Math]::Abs(($m.WorkT - $m.T) - [Math]::Round(28 * $k)) -le 1)
  }

  # --- zona cliccabile e ingrandimento sul primario
  $cx = [int](($dr.L + $dr.Rt) / 2); $above = [int]($dr.T + 20 * $kp); $inside = [int]($dr.B - 30 * $kp)
  [void][MX]::SetCursorPos($prim.L + 5, 300); Start-Sleep 0.8
  Check "a riposo la zona sopra la capsula non blocca i clic" (-not [MX]::IsDockAt($cx, $above, $dock))
  Check "a riposo la capsula blocca i clic" ([MX]::IsDockAt($cx, $inside, $dock))
  [void][MX]::SetCursorPos($cx, $inside); Start-Sleep 0.9
  Check "col cursore sul Dock la zona estesa blocca i clic" ([MX]::IsDockAt($cx, $above, $dock))
  [void][MX]::SetCursorPos($prim.L + 5, 300); Start-Sleep 1.0
  $items = Cdp "JSON.stringify([...document.querySelectorAll('.dock:not(.dock-ghost) .dock-item')].map(e=>{const b=e.getBoundingClientRect();return [e.getAttribute('data-label')||e.title,b.left+b.width/2,b.top+b.height/2]}))" | ConvertFrom-Json
  $target = $items | Where-Object { $_[0] -eq 'Visual Studio Code' } | Select-Object -First 1
  if (-not $target) { $target = $items | Select-Object -Index ([int]($items.Count / 2)) }
  $px = [int]($dr.L + $target[1] * $kp); $py = [int]($dr.T + $target[2] * $kp)
  [void][MX]::SetCursorPos($px, $py); Start-Sleep 1.0
  $now = Cdp "JSON.stringify([...document.querySelectorAll('.dock:not(.dock-ghost) .dock-item')].map(e=>{const b=e.getBoundingClientRect();return [e.getAttribute('data-label')||e.title,b.left+b.width/2,b.width]}))" | ConvertFrom-Json
  $cursorCss = ($px - $dr.L) / $kp
  $nearest = ($now | Sort-Object { [Math]::Abs($_[1] - $cursorCss) } | Select-Object -First 1)
  $biggest = ($now | Sort-Object { -$_[2] } | Select-Object -First 1)
  Check ("ingrandimento: la icona piu' grande ('{0}', {1:N0} px CSS) e' la piu' vicina al cursore ('{2}')" -f $biggest[0], $biggest[2], $nearest[0]) ($biggest[0] -eq $nearest[0] -and $biggest[2] -gt 75)
  [void][MX]::SetCursorPos($prim.L + 5, 300); Start-Sleep 0.5

  # --- sfondo sfocato allineato sul primario (finestra a strisce dietro il Dock)
  $script:stripes = Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'test-stripes-window.ps1') -PassThru -WindowStyle Hidden
  Start-Sleep 4
  $blurOn = $false; $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 8) { if ((Cdp "document.documentElement.getAttribute('data-blur')") -eq 'on') { $blurOn = $true; break }; Start-Sleep -Milliseconds 300 }
  Check "sfocatura attiva (arrivano i fotogrammi dello sfondo)" $blurOn
  if ($blurOn) {
    Start-Sleep 1.5
    $png = Join-Path $env:TEMP "dock-mixed-$($prim.Scala)-$($sec.Scala).png"
    $inject = "(()=>{const s=document.createElement('style');s.id='t';s.textContent='.dock-blur{filter:none!important}.dock::before{display:none!important}.dock-item,.dock-separator{visibility:hidden!important}';document.head.appendChild(s)})()"
    $inner = [double](& node $cdp shot $png $inject)
    $cap = Cdp "JSON.stringify((()=>{const b=document.querySelector('.dock:not(.dock-ghost)').getBoundingClientRect();return [b.left,b.top,b.width,b.height]})())" | ConvertFrom-Json
    [void](& node $cdp eval "document.getElementById('t')?.remove()")
    $img = [Drawing.Bitmap]::FromFile($png); $kk = $img.Width / $inner
    $sw = [int]($cap[2] * $kp); $sh = [int]($cap[3] * $kp)
    $scr = New-Object Drawing.Bitmap $sw, $sh
    $g = [Drawing.Graphics]::FromImage($scr); $g.CopyFromScreen([int]($dr.L + $cap[0] * $kp), [int]($dr.T + $cap[1] * $kp), 0, 0, $scr.Size); $g.Dispose()
    $ok = 0; $tot = 0
    try {
      for ($y = 14; $y -lt $cap[3] - 14; $y += 3) { for ($x = 40; $x -lt $cap[2] - 40; $x += 3) {
        $sx = [Math]::Min([int]($x * $kp), $scr.Width - 1); $sy = [Math]::Min([int]($y * $kp), $scr.Height - 1)
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
  $script:rows += [pscustomobject]@{ Fase = $label; Primario = "$($prim.Scala)%"; Secondario = "$($sec.Scala)%"; Esito = $esito }
}

# ---------------------------------------------------------------------------------------------
$mons = @(Get-Monitors)
$primM = $mons | Where-Object Primario | Select-Object -First 1
$secM = $mons | Where-Object { -not $_.Primario } | Select-Object -First 1
if (-not $secM) { Write-Host "Serve un secondo monitor: ne e' collegato uno solo." -ForegroundColor Yellow; exit 0 }
$origP = $primM.Scala; $origS = $secM.Scala
Write-Host "Monitor primario $($primM.Gdi): $origP%.  Secondario $($secM.Gdi): $origS%."
$okS = @(Get-MonitorScaleRange $secM.Gdi) -contains $SecondaryScale
$okP = @(Get-MonitorScaleRange $primM.Gdi) -contains $PrimaryScale
if (-not $okS -or -not $okP) { Write-Host "Scale non supportate (primario $PrimaryScale%: $okP, secondario $SecondaryScale%: $okS)." -ForegroundColor Yellow; exit 0 }
$deads = @(
  (Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $origP, '-After', 900, '-Gdi', $primM.Gdi -PassThru -WindowStyle Hidden),
  (Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $origS, '-After', 900, '-Gdi', $secM.Gdi -PassThru -WindowStyle Hidden)
)
Write-Host "Reti di sicurezza attive: fra 15 minuti rimettono primario $origP% e secondario $origS% se serve."
try {
  Cleanup-Eos; Start-Sleep 1
  $p = Start-Eos

  Write-Host "`nCambio la scala del SECONDARIO a $SecondaryScale% ..." -ForegroundColor Yellow
  $got = Set-MonitorScale $secM.Gdi $SecondaryScale; if ($got -ne $SecondaryScale) { throw "il secondario ha dato $got% invece di $SecondaryScale%" }
  Start-Sleep 6
  if ($origP -eq $SecondaryScale) { Write-Host "(primario e secondario avrebbero la stessa scala: la fase prova poco)" -ForegroundColor Yellow }
  Run-Checks 'dal vivo: cambia il secondario'

  Write-Host "`nCambio la scala del PRIMARIO a $PrimaryScale% ..." -ForegroundColor Yellow
  $got = Set-MonitorScale $primM.Gdi $PrimaryScale; if ($got -ne $PrimaryScale) { throw "il primario ha dato $got% invece di $PrimaryScale%" }
  Start-Sleep 6
  Run-Checks 'dal vivo: cambia il primario'

  Cleanup-Eos; Start-Sleep 3
  $p = Start-Eos
  Run-Checks 'avvio a freddo con scale diverse'
}
finally {
  Stop-Stripes
  Cleanup-Eos
  Write-Host "`nRipristino le scale originali (primario $origP%, secondario $origS%) ..." -ForegroundColor Yellow
  foreach ($pair in @(@($primM.Gdi, $origP), @($secM.Gdi, $origS))) {
    try { $back = Set-MonitorScale $pair[0] $pair[1]; Write-Host "  $($pair[0]): ora $back%" } catch { Write-Host "  RIPRISTINO FALLITO su $($pair[0]): $_ (la rete di sicurezza riprovera')" -ForegroundColor Red; $fail++ }
  }
  $allBack = (@(Get-Monitors) | Where-Object { ($_.Primario -and $_.Scala -eq $origP) -or (-not $_.Primario -and $_.Scala -eq $origS) }).Count -eq 2
  if ($allBack) { $deads | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } }
  Start-Sleep 3
  $wf = [MX]::Work(); Write-Host "Area di lavoro finale (primario): $($wf.T)..$($wf.B)"
}
Write-Host "`nRiepilogo:"; $rows | Format-Table -AutoSize | Out-String | Write-Host
if ($fail -gt 0) { Write-Host "FALLITI: $fail" -ForegroundColor Red; exit 1 } else { Write-Host "Tutto OK" }
