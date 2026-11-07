# Prova il Launcher con DUE MONITOR A SCALE DIVERSE, cambiando davvero la scala di entrambi e
# ripristinandole alla fine (blocco finally + un processo di sicurezza per ogni monitor cambiato).
# Non fa clic: sposta il cursore e preme Alt+Spazio/Esc (tasti veri, solo se il Launcher e a fuoco).
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte, sul monitor interessato.
#
# Uso: powershell -File scripts\test-launcher-dpi.ps1 [-Exe percorso\eosos.exe] [-SecondaryScale 150] [-PrimaryScale 125]
#
# Fasi (scale del primario / del secondario): 1) dal vivo cambia il secondario, 2) dal vivo cambia
# il primario, 3) avvio a freddo con le due scale diverse. In ogni fase il Launcher si apre due volte:
# col cursore sul primario e col cursore sul secondario, e si controlla che segua la scala del
# monitor su cui si apre: larghezza, centratura, un quinto dall'alto, rapporto di pixel della pagina,
# altezza con i risultati e sfondo sfocato.
param([string]$Exe, [int]$SecondaryScale = 150, [int]$PrimaryScale = 125)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class LD {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static IntPtr Find(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void AltSpace() { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0; $script:rows = @()
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; $p = Start-Process $exe -PassThru; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 9; $p }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $o = & node $cdp eval $expr; if ($o) { $o | ConvertFrom-Json } }
function IsOpen { $h = [LD]::Find('EosOS Launcher'); $h -ne [IntPtr]::Zero -and [LD]::IsWindowVisible($h) }
function Focused { $h = [LD]::Find('EosOS Launcher'); $h -ne [IntPtr]::Zero -and [LD]::GetForegroundWindow() -eq $h }
function TypeChars($text) { foreach ($c in $text.ToCharArray()) { if (Focused) { [LD]::Key([byte][char]$c.ToString().ToUpper()) }; Start-Sleep -Milliseconds 60 } }

function Open-On($mon) {
  [void][LD]::SetCursorPos([int](($mon.L + $mon.R) / 2), [int](($mon.T + $mon.B) / 2))
  Start-Sleep -Milliseconds 400
  [LD]::AltSpace(); Start-Sleep -Milliseconds 1200
}
function Close-Launcher { if (Focused) { [LD]::Key(0x1B) }; Start-Sleep -Milliseconds 600 }

function Run-Checks([string]$label) {
  $mons = @(Get-Monitors)
  $desc = ($mons | ForEach-Object { "$($_.Gdi)=$($_.Scala)%" }) -join ' '
  Write-Host "`n=== $label : $desc ===" -ForegroundColor Cyan
  $failBefore = $script:fail
  Check 'finestra del Launcher presente' ([LD]::Find('EosOS Launcher') -ne [IntPtr]::Zero)
  foreach ($m in $mons) {
    $k = $m.Scala / 100.0
    Write-Host "  -- aperto con il cursore sul $($m.Gdi) ($($m.Scala)%)" -ForegroundColor DarkCyan
    Open-On $m
    Check 'aperto' (IsOpen)
    Check 'in primo piano' (Focused)
    $h = [LD]::Find('EosOS Launcher'); $r = New-Object LD+R; [void][LD]::GetWindowRect($h, [ref]$r)
    $w = $r.Rt - $r.L
    $workW = $m.WorkR - $m.WorkL; $workH = $m.WorkB - $m.WorkT
    $expW = [Math]::Round(680 * $k)
    Check "larghezza = round(680 x $k) = $expW" ([Math]::Abs($w - $expW) -le 1) "(e' $w)"
    Check 'centrato sul monitor di apertura' ([Math]::Abs((($r.L + $r.Rt) / 2) - ($m.WorkL + $workW / 2)) -le 2)
    Check 'a un quinto dell''area di lavoro dall''alto' ([Math]::Abs($r.T - ($m.WorkT + [Math]::Floor($workH / 5))) -le 2) "(T=$($r.T), atteso $($m.WorkT + [Math]::Floor($workH / 5)))"
    Check 'tutto dentro il monitor di apertura' ($r.L -ge $m.L -and $r.Rt -le $m.R -and $r.T -ge $m.T -and $r.B -le $m.B)
    $dpr = [double](Js 'window.devicePixelRatio')
    Check "rapporto di pixel della pagina = $k" ([Math]::Abs($dpr - $k) -lt 0.01) "(e' $dpr)"
    TypeChars 'cal'; Start-Sleep -Milliseconds 1200
    $rows = [int](Js "document.querySelectorAll('.launcher-item').length")
    [void][LD]::GetWindowRect($h, [ref]$r)
    if ($rows -gt 0) {
      $expH = [Math]::Round([Math]::Min(72 + 16 + $rows * 52, 640) * $k)
      Check "altezza = round(CSS $([Math]::Min(72 + 16 + $rows * 52, 640)) x $k) = $expH con $rows righe" ([Math]::Abs(($r.B - $r.T) - $expH) -le 2) "(e' $($r.B - $r.T))"
    } else { Check 'la ricerca da risultati' $false }
    Check 'sfondo sfocato attivo' ((Js "document.documentElement.getAttribute('data-blur')") -eq 'on')
    $cv = Js "(function(){var c=document.querySelector('.launcher-blur');return c.width+'x'+c.height})()"
    $expCw = [Math]::Round((680 + 56) * $k / 2)
    Check "canvas dello sfondo largo ~$expCw px (fotogramma ridotto di 2)" ([Math]::Abs([int]($cv.Split('x')[0]) - $expCw) -le 3) "(e' $cv)"
    Close-Launcher
    Check 'chiuso' (-not (IsOpen))
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
$lj = Join-Path $env:LOCALAPPDATA 'EosOS\launcher.json'
$ljBackup = $null; if (Test-Path $lj) { $ljBackup = Get-Content $lj -Raw }
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
  Cleanup-Eos
  Write-Host "`nRipristino le scale originali (primario $origP%, secondario $origS%) ..." -ForegroundColor Yellow
  foreach ($pair in @(@($primM.Gdi, $origP), @($secM.Gdi, $origS))) {
    try { $back = Set-MonitorScale $pair[0] $pair[1]; Write-Host "  $($pair[0]): ora $back%" } catch { Write-Host "  RIPRISTINO FALLITO su $($pair[0]): $_ (la rete di sicurezza riprovera')" -ForegroundColor Red; $script:fail++ }
  }
  $allBack = (@(Get-Monitors) | Where-Object { ($_.Primario -and $_.Scala -eq $origP) -or (-not $_.Primario -and $_.Scala -eq $origS) }).Count -eq 2
  if ($allBack) { $deads | ForEach-Object { Stop-Process -Id $_.Id -Force -ErrorAction SilentlyContinue } }
  if ($ljBackup) { Set-Content -Path $lj -Value $ljBackup -Encoding UTF8 -NoNewline } else { Remove-Item $lj -ErrorAction SilentlyContinue }
  Start-Sleep 3
}
Write-Host "`nRiepilogo:"; $script:rows | Format-Table -AutoSize | Out-String | Write-Host
if ($script:fail -gt 0) { Write-Host "FALLITI: $($script:fail)" -ForegroundColor Red; exit 1 } else { Write-Host "Tutto OK" }
