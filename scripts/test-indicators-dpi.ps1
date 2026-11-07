# Prova gli indicatori e il loro pannello con DUE MONITOR A SCALE DIVERSE, cambiando davvero la scala
# di entrambi e ripristinandole alla fine (blocco finally + un processo di sicurezza per monitor).
# Stato degli indicatori SIMULATO (build di prova). Clic e tasti veri solo dopo aver verificato il
# bersaglio (la barra di EosOS o il pannello in primo piano).
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte, sul monitor interessato.
#
# Uso: powershell -File scripts\test-indicators-dpi.ps1 [-Exe percorso\eosos.exe] [-SecondaryScale 150] [-PrimaryScale 125]
#
# Fasi: 1) dal vivo cambia il secondario, 2) dal vivo cambia il primario, 3) avvio a freddo. In ogni
# fase, per ciascun monitor: la barra mostra i tre indicatori con la misura giusta, un clic apre il
# pannello alla scala di QUEL monitor (larghezza, posizione sotto l'indicatore, dentro lo schermo,
# rapporto di pixel della pagina) e Esc lo chiude.
param([string]$Exe, [int]$SecondaryScale = 150, [int]$PrimaryScale = 125)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$bytes = [System.IO.File]::ReadAllBytes($exe)
if (-not [System.Text.Encoding]::GetEncoding(28591).GetString($bytes).Contains('EOSOS_FAKE_INDICATORS')) { Write-Host 'Serve una build di prova (feature test-hooks).'; exit 2 }
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class ID {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Esc() { keybd_event(0x1B, 0, 0, UIntPtr.Zero); keybd_event(0x1B, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0; $script:rows = @()
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function PJs([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
$fake = Join-Path $env:TEMP ('eosos-fake-dpi-' + (Get-Random -Maximum 99999) + '.json')
[System.IO.File]::WriteAllText($fake, '{"battery":{"percent":85,"charging":false,"plugged":false,"minutes_left":135},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4},"volume":{"percent":40,"muted":false,"device":"Altoparlanti di prova"}}', (New-Object System.Text.UTF8Encoding($false)))
function Start-Eos {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; $env:EOSOS_FAKE_INDICATORS = $fake
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_FAKE_INDICATORS
  Start-Sleep 10
}
function PanelWin { $w = [ID]::Titled('EosOS Pannello'); if ($w.Count) { $w[0] } else { [IntPtr]::Zero } }
function PanelOpen { $h = PanelWin; $h -ne [IntPtr]::Zero -and [ID]::IsWindowVisible($h) }
# Le pagine della barra non hanno un ordine garantito: si riconosce quella di un monitor dal rapporto di pixel
# (nelle fasi le scale dei due monitor sono diverse).
function PageFor([double]$k) { foreach ($i in 0..3) { $d = PJs 'bar' $i 'window.devicePixelRatio'; if ($null -ne $d -and [Math]::Abs([double]$d - $k) -lt 0.01) { return $i } }; -1 }

function Run-Checks([string]$label) {
  $mons = @(Get-Monitors)
  $desc = ($mons | ForEach-Object { "$($_.Gdi)=$($_.Scala)%" }) -join ' '
  Write-Host "`n=== $label : $desc ===" -ForegroundColor Cyan
  $failBefore = $script:fail
  foreach ($m in $mons) {
    $k = $m.Scala / 100.0
    Write-Host "  -- barra del $($m.Gdi) ($($m.Scala)%)" -ForegroundColor DarkCyan
    $pg = PageFor $k
    Check 'pagina della barra trovata' ($pg -ge 0)
    if ($pg -lt 0) { continue }
    Check 'tre indicatori' (Wait-Until { (PJs 'bar' $pg "document.querySelectorAll('.indicator').length") -eq 3 })
    $bar = [IntPtr]::Zero
    foreach ($b in [ID]::Titled('EosOS')) { $r = New-Object ID+R; [void][ID]::GetWindowRect($b, [ref]$r); if ($r.L -eq $m.L -and $r.T -eq $m.T -and [ID]::IsWindowVisible($b)) { $bar = $b } }
    Check 'finestra della barra del monitor' ($bar -ne [IntPtr]::Zero)
    if ($bar -eq [IntPtr]::Zero) { continue }
    $br = New-Object ID+R; [void][ID]::GetWindowRect($bar, [ref]$br)
    $h = $br.B - $br.T
    Check "altezza della barra = round(28 x $k)" ([Math]::Abs($h - [Math]::Round(28 * $k)) -le 1) "(e' $h)"
    $ic = PJs 'bar' $pg "(function(){var r=document.querySelector('.indicator[data-key=volume]').getBoundingClientRect();return [r.left, r.top, r.width, r.height, r.bottom]})()"
    Check 'gli indicatori stanno dentro la barra' (($ic[4] * $k) -le ($h + 1)) ("fondo " + [Math]::Round($ic[4] * $k, 1) + " px, barra $h")
    Check 'misura CSS dell indicatore invariata (22 px)' ([Math]::Abs($ic[3] - 22) -lt 0.6) ("e' " + $ic[3])
    # clic vero sull'indicatore audio
    $cx = [int]($br.L + ($ic[0] + $ic[2] / 2) * $k); $cy = [int]($br.T + ($ic[1] + $ic[3] / 2) * $k)
    [void][ID]::SetCursorPos($cx, $cy); Start-Sleep -Milliseconds 400
    $under = ([ID]::Root($cx, $cy) -eq $bar)
    Check 'sotto il cursore c e la barra di EosOS (si clicca solo se e vero)' $under
    if (-not $under) { continue }
    [ID]::Click(); Start-Sleep -Milliseconds 1200
    Check 'il pannello si apre' (PanelOpen)
    $pw = PanelWin; $pr = New-Object ID+R; [void][ID]::GetWindowRect($pw, [ref]$pr)
    $expW = [Math]::Round(300 * $k)
    Check "larghezza = round(300 x $k) = $expW" ([Math]::Abs(($pr.Rt - $pr.L) - $expW) -le 1) ("(e' " + ($pr.Rt - $pr.L) + ")")
    $expTop = $br.T + $ic[1] * $k + $ic[3] * $k + [Math]::Round(6 * $k)
    Check 'subito sotto l indicatore' ([Math]::Abs($pr.T - $expTop) -le 3) ("(T=$($pr.T), atteso " + [Math]::Round($expTop) + ")")
    Check 'tutto dentro il monitor della barra cliccata' ($pr.L -ge $m.L -and $pr.Rt -le $m.R -and $pr.T -ge $m.T -and $pr.B -le $m.B)
    $dpr = [double](PJs 'panel' 0 'window.devicePixelRatio')
    Check "rapporto di pixel del pannello = $k" ([Math]::Abs($dpr - $k) -lt 0.01) "(e' $dpr)"
    $bh = [double](PJs 'panel' 0 "document.querySelector('.panel-body').getBoundingClientRect().height")
    Check 'altezza della finestra = contenuto x scala' ([Math]::Abs(($pr.B - $pr.T) - [Math]::Ceiling($bh) * $k) -le 3) ("(finestra " + ($pr.B - $pr.T) + ", contenuto $bh CSS)")
    Check 'sfondo sfocato attivo' (Wait-Until { (PJs 'panel' 0 "document.documentElement.getAttribute('data-blur')") -eq 'on' } 4)
    if ([ID]::GetForegroundWindow() -eq $pw) { [ID]::Esc() }
    Start-Sleep -Milliseconds 700
    Check 'Esc lo chiude' (-not (PanelOpen))
  }
  $esito = if ($script:fail -eq $failBefore) { 'OK' } else { 'FALLITO' }
  $script:rows += [pscustomobject]@{ Fase = $label; Scale = $desc; Esito = $esito }
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
  Start-Eos

  Write-Host "`nCambio la scala del SECONDARIO a $SecondaryScale% ..." -ForegroundColor Yellow
  $got = Set-MonitorScale $secM.Gdi $SecondaryScale; if ($got -ne $SecondaryScale) { throw "il secondario ha dato $got% invece di $SecondaryScale%" }
  Start-Sleep 6
  Run-Checks 'dal vivo: cambia il secondario'

  Write-Host "`nCambio la scala del PRIMARIO a $PrimaryScale% ..." -ForegroundColor Yellow
  $got = Set-MonitorScale $primM.Gdi $PrimaryScale; if ($got -ne $PrimaryScale) { throw "il primario ha dato $got% invece di $PrimaryScale%" }
  Start-Sleep 6
  Run-Checks 'dal vivo: cambia il primario'

  Cleanup-Eos; Start-Sleep 3
  Start-Eos
  Run-Checks 'avvio a freddo con scale diverse'
}
finally {
  Cleanup-Eos
  Remove-Item $fake -ErrorAction SilentlyContinue
  Write-Host "`nRipristino le scale originali (primario $origP%, secondario $origS%) ..." -ForegroundColor Yellow
  foreach ($pair in @(@($primM.Gdi, $origP), @($secM.Gdi, $origS))) {
    try { $back = Set-MonitorScale $pair[0] $pair[1]; Write-Host "  $($pair[0]): ora $back%" } catch { Write-Host "  RIPRISTINO FALLITO su $($pair[0]): $_ (la rete di sicurezza riprovera')" -ForegroundColor Red; $script:fail++ }
  }
  $allBack = (@(Get-Monitors) | Where-Object { ($_.Primario -and $_.Scala -eq $origP) -or (-not $_.Primario -and $_.Scala -eq $origS) }).Count -eq 2
  if ($allBack) { $deads | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } }
  Start-Sleep 3
}
Write-Host "`nRiepilogo:"; $script:rows | Format-Table -AutoSize | Out-String | Write-Host
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host "Tutto OK" }
