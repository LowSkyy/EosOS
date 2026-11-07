# Prova le operazioni CHE MODIFICANO dei desktop virtuali con il backend interno (M5): spostare una finestra su un
# altro desktop, rinominare e rimuovere un desktop. E' l'unica prova che esercita davvero gli indici dei metodi di
# scrittura (la verifica di coerenza all'avvio usa solo metodi di lettura).
#
# ATTENZIONE: cambia lo schermo reale (crea un desktop, ci passa, ne cambia il nome, lo rimuove) e sposta una finestra
# di PROVA (una Calcolatrice nuova, mai una dell'utente). Si esegue SOLO con l'ok esplicito dell'utente e con
# -ConfermatoDallUtente. Non e stata eseguita dall'assistente.
#
# Sicurezze: si ferma se il backend interno non e attivo; rimuove SOLO il desktop creato qui (id diverso da quelli
# dell'inizio) e solo dopo aver riportato la Calcolatrice di prova sul primo desktop; se qualcosa non torna lo dice e
# lascia le istruzioni per ripulire a mano (Win+Ctrl+F4 sul desktop di prova). Niente SendKeys.
#
# Uso: powershell -File scripts\test-vdesk-internal.ps1 -ConfermatoDallUtente [-Exe percorso\eosos.exe]
param([switch]$ConfermatoDallUtente, [string]$Exe)
$ErrorActionPreference = 'Stop'
if (-not $ConfermatoDallUtente) { Write-Host 'Questa prova crea, rinomina e rimuove un desktop virtuale e sposta una finestra di prova: serve -ConfermatoDallUtente (ok esplicito).'; exit 2 }
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class VI {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string t) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == t && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Down() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); }
  public static void Up() { mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void RightClick() { mouse_event(8, 0, 0, 0, UIntPtr.Zero); mouse_event(16, 0, 0, 0, UIntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Desktops { @(Inv 'mc_desktops') }
function Open-Mc { [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8 }
function McIdx([int]$m) { foreach ($i in 0..4) { $v = Js 'mc' $i "new URLSearchParams(location.search).get('m')"; if ($null -ne $v -and [int]$v -eq $m) { return $i } }; -1 }
function CellJs($id) { "Array.from(document.querySelectorAll('.mc-cell')).find(function(e){return e.dataset.id==='$id'})" }
function Page-With-Cell([long]$id) { foreach ($m in 0..3) { $i = McIdx $m; if ($i -ge 0) { $c = Js 'mc' $i "(function(){return $(CellJs $id)?1:null})()"; if ($c) { return $i } } }; -1 }
function Page-Any { foreach ($m in 0..3) { $i = McIdx $m; if ($i -ge 0) { return $i } }; -1 }
function Origin($i) { @([int](Js 'mc' $i 'window.screenX'), [int](Js 'mc' $i 'window.screenY')) }
function Pt($i, [string]$expr) { $o = Origin $i; $c = Js 'mc' $i $expr; @([int]($o[0] + $c[0]), [int]($o[1] + $c[1])) }
function Click-Js($i, [string]$elemExpr) {
  $p = Pt $i "(function(){var r=($elemExpr).getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  [void][VI]::SetCursorPos($p[0], $p[1]); Start-Sleep -Milliseconds 350
  if (@([VI]::Titled('EosOS Mission Control')) -contains [VI]::Root($p[0], $p[1])) { [VI]::Click(); return $true }
  Write-Host '  (clic saltato: sotto il cursore non c e la panoramica)' -ForegroundColor Yellow; $false
}
function Drag-Cell-To-Chip($i, [long]$id, [int]$chipIndex) {
  $a = Pt $i "(function(){var r=$(CellJs $id).getBoundingClientRect();return [r.left+r.width/2, r.top+r.height*0.4]})()"
  $b = Pt $i "(function(){var r=document.querySelector('.mc-space[data-index=\`"$chipIndex\`"]').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  [void][VI]::SetCursorPos($a[0], $a[1]); Start-Sleep -Milliseconds 350
  $o = Origin $i
  $under = Js 'mc' $i "(function(){var e=document.elementFromPoint($($a[0] - $o[0]), $($a[1] - $o[1]));var c=e&&e.closest('.mc-cell');return c?c.dataset.id:null})()"
  if ("$under" -ne "$id") { Write-Host '  (trascinamento saltato: sotto il cursore non c e la cella di prova)' -ForegroundColor Yellow; return $false }
  [VI]::Down()
  foreach ($s in 1..8) { [void][VI]::SetCursorPos([int]($a[0] + ($b[0] - $a[0]) * $s / 8), [int]($a[1] + ($b[1] - $a[1]) * $s / 8)); Start-Sleep -Milliseconds 60 }
  Start-Sleep -Milliseconds 200; [VI]::Up(); Start-Sleep -Milliseconds 900
  $true
}
$created = $null; $calc = [IntPtr]::Zero; $startIds = @()
try {
  Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep 1
  $before = [VI]::Visible()
  Start-Process 'calc.exe' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $calc -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; foreach ($h in [VI]::Visible()) { if ($before -notcontains $h -and ([VI]::Title([IntPtr]$h) -like '*Calcolatrice*')) { $calc = [IntPtr]$h } } }
  if ($calc -eq [IntPtr]::Zero) { throw 'Calcolatrice di prova non trovata' }
  $calcId = $calc.ToInt64()
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 11
  $rep = Inv 'mc_vd_report'
  if (-not $rep.caps.internal) { Write-Host "Il backend interno non e attivo ($($rep.reason)): niente da provare."; exit 3 }
  $start = Desktops; $startIds = @($start | ForEach-Object { $_.id }); $startCur = ($start | Where-Object { $_.current }).id
  Write-Host "Desktop all inizio: $($start.Count)"

  Write-Host 'A. si crea un desktop di prova'
  [void](Open-Mc); $pg = Page-Any
  Check 'clic sul "+"' (Click-Js $pg "document.querySelector('.mc-space-new')")
  Start-Sleep -Seconds 2
  $now = Desktops
  $new = @($now | Where-Object { $startIds -notcontains $_.id })
  Check 'un desktop in piu, ed e quello corrente' (($now.Count -eq $start.Count + 1) -and ($new.Count -eq 1) -and $new[0].current)
  if ($new.Count -ne 1) { throw 'il desktop di prova non risulta creato: mi fermo' }
  $created = $new[0].id

  Write-Host 'B. si torna al primo desktop dalla fascia'
  [void](Open-Mc); $pg = Page-Any
  Check 'clic sul riquadro 1' (Click-Js $pg "document.querySelectorAll('.mc-space')[0]")
  Start-Sleep -Seconds 2
  Check 'il desktop corrente e il primo' ((Desktops | Where-Object { $_.current }).id -eq $startCur)

  Write-Host 'C. la Calcolatrice di prova si sposta sul desktop di prova (trascinando la cella sul riquadro)'
  $idx = @((Desktops | ForEach-Object { $_.id })).IndexOf($created)
  [void](Open-Mc); $pg = Page-With-Cell $calcId
  Check 'la Calcolatrice di prova e nella panoramica del primo desktop' ($pg -ge 0)
  if ($pg -ge 0) {
    Check 'trascinamento sul riquadro del desktop di prova' (Drag-Cell-To-Chip $pg $calcId $idx)
    Check 'la cella sparisce dalla panoramica (non e piu sul desktop corrente)' (Wait-Until { (Page-With-Cell $calcId) -lt 0 } 6)
    Check 'la panoramica e rimasta aperta' ((Inv 'mc_state').phase -eq 'open')
  }
  [void](Inv 'mc_close'); Start-Sleep -Milliseconds 600

  Write-Host 'D. si passa al desktop di prova: la Calcolatrice e li'
  [void](Open-Mc); $pg = Page-Any
  Check 'clic sul riquadro del desktop di prova' (Click-Js $pg "document.querySelectorAll('.mc-space')[$idx]")
  Start-Sleep -Seconds 2
  Check 'il desktop corrente e quello di prova' ((Desktops | Where-Object { $_.current }).id -eq $created)
  [void](Open-Mc)
  Check 'la Calcolatrice di prova compare nella panoramica di questo desktop' ((Page-With-Cell $calcId) -ge 0)

  Write-Host 'E. rinomina del desktop di prova (clic destro sul riquadro, nome, Invio)'
  $pg = Page-Any
  $p = Pt $pg "(function(){var r=document.querySelectorAll('.mc-space')[$idx].getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  [void][VI]::SetCursorPos($p[0], $p[1]); Start-Sleep -Milliseconds 350
  if (@([VI]::Titled('EosOS Mission Control')) -contains [VI]::Root($p[0], $p[1])) {
    [VI]::RightClick(); Start-Sleep -Milliseconds 500
    Check 'compare il campo del nome' ((Js 'mc' $pg "document.querySelectorAll('.mc-space-input').length") -eq 1)
    foreach ($c in 'PROVA'.ToCharArray()) { [VI]::Key([byte][char]$c); Start-Sleep -Milliseconds 60 }
    [VI]::Key(0x0D); Start-Sleep -Milliseconds 1200
    Check 'il nome e cambiato (registro)' ((Desktops | Where-Object { $_.id -eq $created }).name -like '*Desktop*PROVA*' -or (Desktops | Where-Object { $_.id -eq $created }).name -eq 'PROVA' -or (Desktops | Where-Object { $_.id -eq $created }).name -like '*PROVA')
  }

  Write-Host 'F. la Calcolatrice torna sul primo desktop'
  $pg = Page-With-Cell $calcId
  if ($pg -ge 0) {
    Check 'trascinamento sul riquadro 1' (Drag-Cell-To-Chip $pg $calcId 0)
    Check 'la cella sparisce da questo desktop' (Wait-Until { (Page-With-Cell $calcId) -lt 0 } 6)
  }
  [void](Inv 'mc_close'); Start-Sleep -Milliseconds 600
  [void](Open-Mc); $pg = Page-Any
  Check 'clic sul riquadro 1' (Click-Js $pg "document.querySelectorAll('.mc-space')[0]")
  Start-Sleep -Seconds 2
  [void](Open-Mc)
  Check 'sul primo desktop la Calcolatrice di prova c e di nuovo' ((Page-With-Cell $calcId) -ge 0)

  Write-Host 'G. si rimuove il desktop di prova con la x (e solo quello)'
  $pg = Page-Any
  $idx = @((Desktops | ForEach-Object { $_.id })).IndexOf($created)
  Check 'il desktop da rimuovere e quello creato qui' ($idx -ge 0 -and $startIds -notcontains $created)
  if ($idx -ge 0) {
    $o = Origin $pg
    # La x compare al passaggio del mouse sul riquadro: ci si porta sopra e poi sulla x.
    $chip = Pt $pg "(function(){var r=document.querySelectorAll('.mc-space')[$idx].getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
    [void][VI]::SetCursorPos($chip[0], $chip[1]); Start-Sleep -Milliseconds 400
    Check 'clic sulla x' (Click-Js $pg "document.querySelectorAll('.mc-space-remove')[0]")
    Check 'i desktop sono tornati quelli di partenza' (Wait-Until { (Desktops).Count -eq $start.Count } 6)
    $created = $null
  }
  Check 'la Calcolatrice di prova e ancora li' ([VI]::IsWindow($calc))
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  try { [void](Inv 'mc_close') } catch {}
  try {
    if ($created) { Write-Host "ATTENZIONE: il desktop di prova ($created) esiste ancora: chiudilo a mano (Win+Ctrl+F4 mentre ci sei sopra)." -ForegroundColor Yellow; $script:fail++ }
  } catch {}
  if ($calc -ne [IntPtr]::Zero -and [VI]::IsWindow($calc)) { [void][VI]::PostMessage($calc, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Start-Sleep 1
  Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
