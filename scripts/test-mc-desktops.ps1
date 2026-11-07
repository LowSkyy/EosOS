# Prova la fascia dei desktop virtuali di Mission Control (M4) CAMBIANDO DAVVERO il desktop: crea un desktop di prova
# con il "+" della fascia, ci passa, torna al primo con il suo riquadro e poi RIMUOVE il desktop di prova.
#
# ATTENZIONE: questa prova cambia lo schermo reale (Windows anima il passaggio fra desktop) e crea/rimuove un
# desktop virtuale. Si esegue SOLO con l'ok esplicito dell'utente e con il parametro -ConfermatoDallUtente.
# Non e stata eseguita dall'assistente. Sicurezze: rimuove SOLO un desktop il cui id non c'era all'inizio, mai un
# desktop dell'utente; se qualcosa non torna si ferma e lo dice. Niente SendKeys.
#
# Uso: powershell -File scripts\test-mc-desktops.ps1 -ConfermatoDallUtente [-Exe percorso\eosos.exe]
param([switch]$ConfermatoDallUtente, [string]$Exe)
$ErrorActionPreference = 'Stop'
if (-not $ConfermatoDallUtente) { Write-Host 'Questa prova crea e rimuove un desktop virtuale e passa fra desktop: serve -ConfermatoDallUtente (ok esplicito).'; exit 2 }
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class DK {
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static IntPtr[] Titled(string t) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (sb.ToString() == t && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  // Win+Ctrl+F4: chiude il desktop virtuale corrente (le sue finestre passano al precedente).
  public static void CloseDesktop() { keybd_event(0x5B, 0, 0, UIntPtr.Zero); keybd_event(0x11, 0, 0, UIntPtr.Zero); keybd_event(0x73, 0, 0, UIntPtr.Zero); keybd_event(0x73, 0, 2, UIntPtr.Zero); keybd_event(0x11, 0, 2, UIntPtr.Zero); keybd_event(0x5B, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Desktops { @(Inv 'mc_desktops') }
function McIdx([int]$m) { foreach ($i in 0..4) { $v = Js 'mc' $i "new URLSearchParams(location.search).get('m')"; if ($null -ne $v -and [int]$v -eq $m) { return $i } }; -1 }
function Click-Chip([int]$m, [string]$selector) {
  $i = McIdx $m; if ($i -lt 0) { return $false }
  $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  $c = Js 'mc' $i "(function(){var r=$selector.getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $x = [int]($sx + $c[0]); $y = [int]($sy + $c[1])
  [void][DK]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 400
  $ok = (@([DK]::Titled('EosOS Mission Control')) -contains [DK]::Root($x, $y))
  if ($ok) { [DK]::Click() } else { Write-Host '  (clic saltato: sotto il cursore non c e la panoramica)' -ForegroundColor Yellow }
  $ok
}
$created = $null
try {
  Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 11
  $start = Desktops; $startIds = @($start | ForEach-Object { $_.id }); $startCur = ($start | Where-Object { $_.current }).id
  Write-Host "Desktop all inizio: $($start.Count)"
  [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8 | Out-Null

  Write-Host 'A. il "+" crea un desktop e ci passa'
  Check 'clic sul "+"' (Click-Chip 0 "document.querySelector('.mc-space-new')")
  Check 'la panoramica si chiude' (Wait-Until { (@([DK]::Titled('EosOS Mission Control'))).Count -eq 0 } 4)
  Start-Sleep -Seconds 2
  $after = Desktops
  Check 'i desktop sono uno in piu' ($after.Count -eq $start.Count + 1)
  $new = @($after | Where-Object { $startIds -notcontains $_.id })
  Check 'il nuovo desktop e uno solo ed e quello corrente' (($new.Count -eq 1) -and $new[0].current)
  if ($new.Count -eq 1) { $created = $new[0].id }

  Write-Host 'B. dalla fascia si torna al primo desktop'
  if ($created) {
    [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8 | Out-Null
    Check 'clic sul riquadro 1' (Click-Chip 0 "document.querySelectorAll('.mc-space')[0]")
    Start-Sleep -Seconds 2
    $cur = (Desktops | Where-Object { $_.current }).id
    Check 'il desktop corrente e di nuovo il primo' ($cur -eq $startCur)
    # E si torna al desktop di prova per poterlo rimuovere.
    [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8 | Out-Null
    $idx = ((Desktops | ForEach-Object { $_.id }).IndexOf($created)) + 1
    Check 'clic sul riquadro del desktop di prova' (Click-Chip 0 "document.querySelectorAll('.mc-space')[$($idx - 1)]")
    Start-Sleep -Seconds 2
    Check 'si e passati al desktop di prova' ((Desktops | Where-Object { $_.current }).id -eq $created)
  }
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  # Rimozione: SOLO il desktop creato qui e SOLO se e quello corrente (Win+Ctrl+F4 chiude il corrente).
  try {
    if ($created) {
      $cur = (Desktops | Where-Object { $_.current }).id
      if ($cur -eq $created) { [DK]::CloseDesktop(); Start-Sleep -Seconds 2 }
      $left = Desktops
      Write-Host "Desktop alla fine: $($left.Count) (all inizio $($startIds.Count))"
      if (@($left | Where-Object { $_.id -eq $created }).Count -gt 0) { Write-Host 'ATTENZIONE: il desktop di prova esiste ancora: chiudilo a mano (Win+Ctrl+F4 mentre ci sei sopra).' -ForegroundColor Yellow; $script:fail++ }
    }
  } catch {}
  Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
