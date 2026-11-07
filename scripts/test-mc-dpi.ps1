# Prova Mission Control con DUE MONITOR A SCALE DIVERSE, cambiando davvero la scala di entrambi e
# ripristinandole alla fine (blocco finally + un processo di sicurezza per monitor). Si usa una Calcolatrice di
# prova NUOVA; la panoramica mostra anche le finestre dell'utente come in uso reale, ma non si toccano, non si
# cliccano e i loro titoli non si stampano. Nessun clic: si apre/chiude con i comandi e con Esc (tasto vero, solo
# se la panoramica e in primo piano). Nessun desktop virtuale viene creato o cambiato.
#
# ATTENZIONE: ogni cambio di scala ridisegna tutte le finestre aperte, sul monitor interessato.
#
# Uso: powershell -File scripts\test-mc-dpi.ps1 [-Exe percorso\eosos.exe] [-SecondaryScale 150] [-PrimaryScale 125]
param([string]$Exe, [int]$SecondaryScale = 150, [int]$PrimaryScale = 125)
$ErrorActionPreference = 'Stop'
. "$PSScriptRoot\lib-dpi.ps1"
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MD {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string t) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == t && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0; $script:rows = @()
function Check($l, $c, $d = '') { if ($c) { Write-Host "  [OK]    $l $d" } else { Write-Host "  [FALLITO] $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Overlays { @([MD]::Titled('EosOS Mission Control')) }
function IsOpen { (Overlays).Count -gt 0 }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 11 }
function Find-New($before, [string]$namePart) { foreach ($h in [MD]::Visible()) { if ($before -notcontains $h -and ([MD]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function CellJs($id) { "Array.from(document.querySelectorAll('.mc-cell')).find(function(e){return e.dataset.id==='$id'})" }
# Le pagine "mc" non hanno un ordine garantito: si riconosce quella di un monitor dal rapporto di pixel (le scale dei due sono diverse).
function PageFor([double]$k) { foreach ($i in 0..4) { $d = Js 'mc' $i 'window.devicePixelRatio'; if ($null -ne $d -and [Math]::Abs([double]$d - $k) -lt 0.01) { return $i } }; -1 }
function Avg-Color($x, $y, $w, $h) {
  $bmp = New-Object Drawing.Bitmap $w, $h
  $g = [Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size); $g.Dispose()
  $sr = 0; $sg = 0; $sb = 0; $n = 0
  for ($yy = 2; $yy -lt $h - 2; $yy += 4) { for ($xx = 2; $xx -lt $w - 2; $xx += 4) { $p = $bmp.GetPixel($xx, $yy); $sr += $p.R; $sg += $p.G; $sb += $p.B; $n++ } }
  $bmp.Dispose()
  [pscustomobject]@{ R = $sr / $n; G = $sg / $n; B = $sb / $n }
}
$calc = [IntPtr]::Zero

function Run-Checks([string]$label) {
  $mons = @(Get-Monitors)
  $desc = ($mons | ForEach-Object { "$($_.Gdi)=$($_.Scala)%" }) -join ' '
  Write-Host "`n=== $label : $desc ===" -ForegroundColor Cyan
  $failBefore = $script:fail
  # Colore della Calcolatrice vera, PRIMA di aprire la panoramica (e' sopra tutto: appena creata).
  $realColor = $null; $rc = New-Object MD+R
  if ([MD]::IsWindow($calc) -and [MD]::GetWindowRect($calc, [ref]$rc) -and ($rc.Rt - $rc.L) -gt 100) { $realColor = Avg-Color ($rc.L + 10) ($rc.T + 40) (($rc.Rt - $rc.L) - 20) (($rc.B - $rc.T) - 50) }
  [void](Inv 'mc_open')
  Check 'la panoramica si apre' (Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8)
  $ovs = Overlays
  Check 'una finestra panoramica per monitor' ($ovs.Count -eq $mons.Count) ("finestre " + $ovs.Count)
  $calcId = $calc.ToInt64()
  foreach ($m in $mons) {
    $k = $m.Scala / 100.0
    Write-Host "  -- monitor $($m.Gdi) ($($m.Scala)%)" -ForegroundColor DarkCyan
    $hit = $false
    foreach ($o in $ovs) { $r = New-Object MD+R; [void][MD]::GetWindowRect($o, [ref]$r); if ($r.L -eq $m.L -and $r.T -eq $m.T -and ($r.Rt - $r.L) -eq ($m.R - $m.L) -and ($r.B - $r.T) -eq ($m.B - $m.T)) { $hit = $true } }
    $dbg = ($ovs | ForEach-Object { $r = New-Object MD+R; [void][MD]::GetWindowRect($_, [ref]$r); "($($r.L),$($r.T),$($r.Rt),$($r.B))" }) -join ' '
    Check 'la finestra panoramica copre esattamente il monitor' $hit ("monitor ($($m.L),$($m.T),$($m.R),$($m.B)); finestre $dbg")
    $pg = PageFor $k
    Check "pagina con rapporto di pixel $k" ($pg -ge 0)
    if ($pg -lt 0) { continue }
    $dim = Js 'mc' $pg "[window.innerWidth * window.devicePixelRatio, window.innerHeight * window.devicePixelRatio]"
    Check 'la pagina riempie il monitor in pixel fisici' (([Math]::Abs($dim[0] - ($m.R - $m.L)) -le 2) -and ([Math]::Abs($dim[1] - ($m.B - $m.T)) -le 2)) ("pagina " + [Math]::Round($dim[0]) + "x" + [Math]::Round($dim[1]))
    $geo = Js 'mc' $pg "(function(){var w=window.innerWidth,h=window.innerHeight,ok=true,n=0,minTop=1e9;document.querySelectorAll('.mc-cell').forEach(function(e){n++;var r=e.getBoundingClientRect();minTop=Math.min(minTop,r.top);if(r.left<59||r.top<139||r.right>w-59||r.bottom>h-59)ok=false});var s=document.querySelector('.mc-spaces');return {inside:ok,n:n,minTop:minTop,barBottom:s?s.getBoundingClientRect().bottom:null,w:w,h:h}})()"
    Check 'celle dentro l area (margine 60, fascia in alto 140: misure CSS, uguali a ogni scala)' ($geo.inside -eq $true) ("celle " + $geo.n)
    Check 'la fascia dei desktop sta sopra le celle' (($null -ne $geo.barBottom) -and ($geo.barBottom -le $geo.minTop)) ("fascia fino a " + $geo.barBottom + ", celle da " + $geo.minTop)
    $chip = Js 'mc' $pg "(function(){var r=document.querySelector('.mc-space').getBoundingClientRect();return [r.width,r.height]})()"
    Check 'riquadro del desktop: misure CSS invariate (72 px di altezza)' ([Math]::Abs($chip[1] - 72) -lt 0.6) ("e' " + $chip[1])
    $bg = Js 'mc' $pg "document.documentElement.getAttribute('data-mc-bg')"
    Check 'sfondo scattato a questa scala' ($bg -eq 'on')
    $cc = Js 'mc' $pg "(function(){var e=$(CellJs $calcId);if(!e)return null;var r=e.getBoundingClientRect();return {x:r.left,y:r.top,w:r.width,h:r.height,card:e.dataset.card}})()"
    if ($cc -and $realColor) {
      Check 'la Calcolatrice di prova ha una miniatura viva (non una scheda)' ($cc.card -eq 'false')
      # Pixel dello schermo = origine del monitor + posizione CSS x scala. Riquadro della miniatura: cella meno margine 8 e fascia 38.
      $tx = [int]($m.L + ($cc.x + 8) * $k + 6); $ty = [int]($m.T + ($cc.y + 8) * $k + 6)
      $tw = [int](($cc.w - 16) * $k - 12); $th = [int](($cc.h - 16 - 38) * $k - 12)
      if ($tw -gt 20 -and $th -gt 20) {
        $thumb = Avg-Color $tx $ty $tw $th
        $d = [Math]::Abs($thumb.R - $realColor.R) + [Math]::Abs($thumb.G - $realColor.G) + [Math]::Abs($thumb.B - $realColor.B)
        Check 'la miniatura sta dove deve a questa scala (colore medio ~ finestra reale)' ($d -lt 120) ("differenza " + [Math]::Round($d))
      }
    } elseif (-not $cc) { Write-Host '  (la Calcolatrice di prova e su un altro monitor)' }
  }
  Check 'miniature vive registrate' ((Inv 'mc_state').thumbs -ge 1)
  # Chiusura con Esc vero se la panoramica e a fuoco, altrimenti con il comando.
  $fg = [MD]::GetForegroundWindow()
  if ((Overlays) -contains $fg) { [MD]::Key(0x1B) } else { [void](Inv 'mc_close') }
  Check 'si chiude' (Wait-Until { -not (IsOpen) -and (Inv 'mc_state').phase -eq 'closed' } 5)
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
  $before = [MD]::Visible()
  Start-Process 'calc.exe' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $calc -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; $calc = Find-New $before 'Calcolatrice' }
  if ($calc -eq [IntPtr]::Zero) { throw 'Calcolatrice di prova non trovata' }
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
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  try { [void](Inv 'mc_close') } catch {}
  if ($calc -ne [IntPtr]::Zero -and [MD]::IsWindow($calc)) { [void][MD]::PostMessage($calc, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Start-Sleep 1
  Cleanup-Eos
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
