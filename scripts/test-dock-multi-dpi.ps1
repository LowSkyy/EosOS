# Un Dock per monitor con DUE MONITOR A SCALE DIVERSE: cambia davvero la scala di entrambi e la ripristina SEMPRE (blocco finally
# + un processo di sicurezza per monitor). Per ogni fase si controlla che ogni Dock stia dentro il PROPRIO monitor, centrato in
# basso, che la sua pagina abbia la scala del proprio monitor e che riceva la sfocatura dello sfondo. Non si clicca niente.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte, sul monitor interessato.
# Uso: powershell -File scripts\test-dock-multi-dpi.ps1 [-Exe percorso\eosos.exe] [-SecondaryScale 150] [-PrimaryScale 125]
param([string]$Exe, [int]$SecondaryScale = 150, [int]$PrimaryScale = 125)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class DD {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  public static string Docks() { string o = ""; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); var t = sb.ToString(); if (IsWindowVisible(h) && t.StartsWith("EosOS Dock")) { R r; GetWindowRect(h, out r); o += r.L + "," + r.T + "," + r.Rt + "," + r.B + ";"; } return true; }, IntPtr.Zero); return o; }
}
'@
$script:fail = 0
$script:rows = @()
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function JsD([int]$i, [string]$expr) { $env:CDP_VIEW = 'dock'; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }; $false }
function Dock-Rects { @(([DD]::Docks() -split ';' | Where-Object { $_ }) | ForEach-Object { $c = $_ -split ','; [pscustomobject]@{ l = [int]$c[0]; t = [int]$c[1]; r = [int]$c[2]; b = [int]$c[3] } }) }
function Start-Eos {
  $env:EOSOS_DOCK_ALL = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 14
}
function Run-Checks([string]$label) {
  Write-Host "=== $label ==="
  $before = $script:fail
  $mons = @(Get-Monitors)
  $docks = Dock-Rects
  Check 'un Dock per monitor' (Wait-Until { (Dock-Rects).Count -eq $mons.Count } 12) "Dock: $((Dock-Rects).Count), monitor: $($mons.Count)"
  $docks = Dock-Rects
  foreach ($m in $mons) {
    $d = $docks | Where-Object { ($_.l + $_.r) / 2 -ge $m.L -and ($_.l + $_.r) / 2 -lt $m.R } | Select-Object -First 1
    $nome = "$($m.Gdi) ($($m.Scala)%)"
    Check "${nome}: c'e' un Dock dentro il proprio monitor" ($null -ne $d -and $d.l -ge $m.L - 2 -and $d.r -le $m.R + 2 -and $d.b -le $m.B + 2) ("Dock " + $d.l + ',' + $d.t + ',' + $d.r + ',' + $d.b + " monitor $($m.L),$($m.T),$($m.R),$($m.B)")
    if ($d) {
      $cx = ($d.l + $d.r) / 2; $mx = ($m.L + $m.R) / 2
      Check "${nome}: centrato in basso" (([math]::Abs($cx - $mx) -le (40 * $m.Scala / 100)) -and ($d.b -ge $m.B - 90 * $m.Scala / 100)) "centro $cx/$mx, fondo $($d.b)/$($m.B)"
      Check "${nome}: ha misure in proporzione alla scala (larghezza fra 400 e 900 px logici)" ((($d.r - $d.l) * 100 / $m.Scala) -ge 400 -and (($d.r - $d.l) * 100 / $m.Scala) -le 1400) ("larghezza fisica " + ($d.r - $d.l))
    }
  }
  # Le pagine: l'insieme delle scale delle due pagine Dock e' quello delle scale dei monitor.
  $dprs = @(0..($mons.Count - 1) | ForEach-Object { [math]::Round([double](JsD $_ 'window.devicePixelRatio') * 100) } | Sort-Object)
  $want = @($mons | ForEach-Object { $_.Scala } | Sort-Object)
  Check 'le pagine dei Dock hanno le scale dei monitor' (($dprs -join ',') -eq ($want -join ',')) "pagine: $($dprs -join ',')  monitor: $($want -join ',')"
  foreach ($i in 0..($mons.Count - 1)) { Check "Dock ${i}: arriva lo sfondo sfocato" (Wait-Until { (JsD $i "document.documentElement.getAttribute('data-blur')") -eq 'on' } 12) }
  $esito = if ($script:fail -eq $before) { 'OK' } else { 'FALLITO' }
  $prim = $mons | Where-Object Primario | Select-Object -First 1; $sec = $mons | Where-Object { -not $_.Primario } | Select-Object -First 1
  $script:rows += [pscustomobject]@{ Fase = $label; Primario = "$($prim.Scala)%"; Secondario = "$($sec.Scala)%"; Esito = $esito }
}

$mons0 = @(Get-Monitors)
$primM = $mons0 | Where-Object Primario | Select-Object -First 1
$secM = $mons0 | Where-Object { -not $_.Primario } | Select-Object -First 1
if (-not $secM) { Write-Host "Serve un secondo monitor: ne e' collegato uno solo." -ForegroundColor Yellow; exit 0 }
$origP = $primM.Scala; $origS = $secM.Scala
Write-Host "Monitor primario $($primM.Gdi): $origP%.  Secondario $($secM.Gdi): $origS%."
$okS = @(Get-MonitorScaleRange $secM.Gdi) -contains $SecondaryScale
$okP = @(Get-MonitorScaleRange $primM.Gdi) -contains $PrimaryScale
if (-not $okS -or -not $okP) { Write-Host "Scale non supportate (primario $PrimaryScale%: $okP, secondario $SecondaryScale%: $okS)." -ForegroundColor Yellow; exit 0 }
$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$deads = @(
  (Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $origP, '-After', 900, '-Gdi', $primM.Gdi -PassThru -WindowStyle Hidden),
  (Start-Process powershell -ArgumentList '-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'dpi-deadman.ps1'), '-Orig', $origS, '-After', 900, '-Gdi', $secM.Gdi -PassThru -WindowStyle Hidden)
)
Write-Host "Reti di sicurezza attive: fra 15 minuti rimettono primario $origP% e secondario $origS% se serve."
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  Start-Eos
  Run-Checks "scale di partenza ($origP% / $origS%)"
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
  Remove-Item env:EOSOS_DOCK_ALL -ErrorAction SilentlyContinue
  Write-Host "`nRipristino le scale originali (primario $origP%, secondario $origS%) ..." -ForegroundColor Yellow
  foreach ($pair in @(@($primM.Gdi, $origP), @($secM.Gdi, $origS))) {
    try { $back = Set-MonitorScale $pair[0] $pair[1]; Write-Host "  $($pair[0]): ora $back%" } catch { Write-Host "  RIPRISTINO FALLITO su $($pair[0]): $_ (la rete di sicurezza riprovera')" -ForegroundColor Red; $script:fail++ }
  }
  $allBack = (@(Get-Monitors) | Where-Object { ($_.Primario -and $_.Scala -eq $origP) -or (-not $_.Primario -and $_.Scala -eq $origS) }).Count -eq 2
  if ($allBack) { $deads | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } }
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
Write-Host "`nRiepilogo:"; $script:rows | Format-Table -AutoSize | Out-String | Write-Host
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host 'Tutto OK' }
