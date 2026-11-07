# Verifica la panoramica di Mission Control (M2) sul sistema vero. Bersagli di TUTTI i clic: solo le
# finestre di prova create qui (una Calcolatrice e un Blocco note NUOVI); prima di ogni clic si controlla
# che sotto il cursore ci sia la panoramica e che l'elemento sia proprio la cella di una finestra di prova.
# Le finestre dell'utente compaiono nella panoramica come in uso reale, ma non si toccano e i loro titoli
# non si stampano. Niente SendKeys: tasti veri solo se la panoramica e in primo piano.
# Di sola lettura sullo schermo: lo screenshot della miniatura e ritagliato SOLO sulla cella di prova.
#
# Uso: powershell -File scripts\test-mc-overlay.ps1 [-Exe percorso\eosos.exe]   (qualunque build)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MO {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == title && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void AltSpace() { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
# Pagina "mc" del monitor m (l'ordine delle pagine nell'elenco di CDP non e garantito).
function McIdx([int]$m) { foreach ($i in 0..4) { $v = Js 'mc' $i "new URLSearchParams(location.search).get('m')"; if ($null -ne $v -and [int]$v -eq $m) { return $i } }; -1 }
function Mc([int]$m, [string]$expr) { $i = McIdx $m; if ($i -ge 0) { Js 'mc' $i $expr } }
function Overlays { @([MO]::Titled('EosOS Mission Control')) }
function IsOpen { (Overlays).Count -gt 0 }
function Overlay-Rect($h) { $r = New-Object MO+R; [void][MO]::GetWindowRect($h, [ref]$r); $r }
function Find-New($before, [string]$namePart) { foreach ($h in [MO]::Visible()) { if ($before -notcontains $h -and ([MO]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function Open-Mc { [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').open -eq $true -and (IsOpen) } 8 }
# Espressione JS che trova la cella di una finestra per id (senza selettori con virgolette).
function CellJs($id) { "Array.from(document.querySelectorAll('.mc-cell')).find(function(e){return e.dataset.id==='$id'})" }
function Cell-Info([int]$m, [long]$id) { Mc $m "(function(){var e=$(CellJs $id);if(!e)return null;var r=e.getBoundingClientRect();return {x:r.left,y:r.top,w:r.width,h:r.height,card:e.dataset.card,min:e.dataset.minimized,hover:e.dataset.hover,dpr:window.devicePixelRatio}})()" }
function Monitor-Of([long]$id) { foreach ($m in 0..3) { if ((McIdx $m) -ge 0) { $c = Cell-Info $m $id; if ($c) { return $m } } }; -1 }
function Screen-Point($m, $c, [double]$fx, [double]$fy) {
  $i = McIdx $m
  $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  [pscustomobject]@{ X = [int]($sx + ($c.x + $c.w * $fx)); Y = [int]($sy + ($c.y + $c.h * $fy)) }
}
function Esc-If-Open { if (IsOpen) { $fg = [MO]::GetForegroundWindow(); if ((Overlays) -contains $fg) { [MO]::Key(0x1B) }; Start-Sleep -Milliseconds 600 } }

$mine = @(); $calc = [IntPtr]::Zero; $note = [IntPtr]::Zero
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [MO]::Visible()
  Write-Host 'Preparazione: una Calcolatrice e un Blocco note NUOVI (le finestre dell utente non si toccano)'
  Start-Process 'calc.exe' | Out-Null; Start-Process 'notepad.exe' | Out-Null
  $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 15 -and ($calc -eq [IntPtr]::Zero -or $note -eq [IntPtr]::Zero)) {
    Start-Sleep -Milliseconds 400
    if ($calc -eq [IntPtr]::Zero) { $calc = Find-New $before 'Calcolatrice' }
    if ($note -eq [IntPtr]::Zero) { $note = Find-New $before 'Blocco note' }
  }
  Check 'finestre di prova create' ($calc -ne [IntPtr]::Zero -and $note -ne [IntPtr]::Zero)
  if ($calc -eq [IntPtr]::Zero -or $note -eq [IntPtr]::Zero) { throw 'finestre di prova non trovate' }
  $mine = @($calc, $note)
  $calcId = $calc.ToInt64(); $noteId = $note.ToInt64()
  # Colore medio della finestra reale (ancora sopra tutto, appena creata) per confrontarlo con la sua miniatura.
  $rc0 = Overlay-Rect $calc
  function Avg-Color($x, $y, $w, $h) {
    $bmp = New-Object Drawing.Bitmap $w, $h
    $g = [Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($x, $y, 0, 0, $bmp.Size); $g.Dispose()
    $sr = 0; $sg = 0; $sb = 0; $n = 0
    for ($yy = 2; $yy -lt $h - 2; $yy += 4) { for ($xx = 2; $xx -lt $w - 2; $xx += 4) { $p = $bmp.GetPixel($xx, $yy); $sr += $p.R; $sg += $p.G; $sb += $p.B; $n++ } }
    $bmp.Dispose()
    [pscustomobject]@{ R = $sr / $n; G = $sg / $n; B = $sb / $n }
  }
  $realColor = Avg-Color ($rc0.L + 10) ($rc0.T + 40) (($rc0.Rt - $rc0.L) - 20) (($rc0.B - $rc0.T) - 50)
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 10

  Write-Host 'A. apertura'
  Check 'chiusa all inizio' (-not (IsOpen) -and ((Inv 'mc_state').open -eq $false))
  Check 'si apre' (Open-Mc)
  $ovs = Overlays
  $nMon = [System.Windows.Forms.Screen]::AllScreens.Length
  Check 'una finestra panoramica per monitor' ($ovs.Count -eq $nMon) ("finestre " + $ovs.Count + ", monitor $nMon")
  $okCover = $true
  foreach ($o in $ovs) { $r = Overlay-Rect $o; $hit = $false; foreach ($s in [System.Windows.Forms.Screen]::AllScreens) { if ($r.L -eq $s.Bounds.X -and $r.T -eq $s.Bounds.Y -and ($r.Rt - $r.L) -eq $s.Bounds.Width -and ($r.B - $r.T) -eq $s.Bounds.Height) { $hit = $true } }; if (-not $hit) { $okCover = $false } }
  Check 'ognuna copre esattamente il suo monitor' $okCover
  $st = Inv 'mc_state'
  Check 'miniature vive registrate' ($st.thumbs -ge 1) ("miniature " + $st.thumbs + ", finestre " + $st.windows)
  $mc = Monitor-Of $calcId
  Check 'la Calcolatrice di prova e in una panoramica' ($mc -ge 0) "monitor $mc"
  $cc = Cell-Info $mc $calcId
  Check 'cella con miniatura viva (non scheda)' ($cc.card -eq 'false' -and $cc.min -eq 'false')
  $cn = Cell-Info $mc $noteId
  Check 'il Blocco note di prova e elencato' ($null -ne (Monitor-Of $noteId))
  Check 'sfondo scattato e disegnato' ((Mc $mc "document.documentElement.getAttribute('data-mc-bg')") -eq 'on')
  $nCells = [int](Mc $mc "document.querySelectorAll('.mc-cell').length")
  Check 'celle in pagina = finestre elencate per quel monitor' ($nCells -ge 1) "celle $nCells"
  $noTitleLeak = [int](Mc $mc "document.querySelectorAll('.mc-cell .mc-label-title').length")
  Check 'ogni cella ha etichetta con titolo' ($noTitleLeak -eq $nCells)

  Write-Host 'B. la miniatura e davvero la finestra (colori)'
  $sp = Screen-Point $mc $cc 0.0 0.0
  # Riquadro della miniatura: la cella meno margine (8 px) e fascia etichetta (38 px).
  $k = [double]$cc.dpr
  $tx = [int]($sp.X + 8 * $k + 6); $ty = [int]($sp.Y + 8 * $k + 6)
  $tw = [int]($cc.w * $k - 16 * $k - 12); $th = [int]($cc.h * $k - 16 * $k - 38 * $k - 12)
  if ($tw -gt 20 -and $th -gt 20) {
    $thumbColor = Avg-Color $tx $ty $tw $th
    $d = [Math]::Abs($thumbColor.R - $realColor.R) + [Math]::Abs($thumbColor.G - $realColor.G) + [Math]::Abs($thumbColor.B - $realColor.B)
    Check 'colore medio della miniatura ~ colore della finestra reale' ($d -lt 120) ("differenza " + [Math]::Round($d) + " (miniatura " + [Math]::Round($thumbColor.R) + "," + [Math]::Round($thumbColor.G) + "," + [Math]::Round($thumbColor.B) + " / reale " + [Math]::Round($realColor.R) + "," + [Math]::Round($realColor.G) + "," + [Math]::Round($realColor.B) + ")")
    if ($env:MC_SHOT) {
      $bmp = New-Object Drawing.Bitmap ([int]($cc.w * $k)), ([int]($cc.h * $k))
      $g = [Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($sp.X, $sp.Y, 0, 0, $bmp.Size); $g.Dispose(); $bmp.Save($env:MC_SHOT + '-cell.png'); $bmp.Dispose()
    }
  } else { Write-Host '  (cella troppo piccola per il confronto dei colori)' -ForegroundColor Yellow }

  Write-Host 'C. passaggio del mouse'
  $p = Screen-Point $mc $cc 0.5 0.4
  [void][MO]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 500
  $under = [MO]::Root($p.X, $p.Y)
  Check 'sotto il cursore c e la panoramica' (($ovs | Where-Object { $_ -eq $under }).Count -eq 1)
  Check 'la cella e evidenziata' ((Cell-Info $mc $calcId).hover -eq 'true')

  Write-Host 'D. Esc chiude e la pagina dimentica'
  $fg = [MO]::GetForegroundWindow()
  Check 'la panoramica e in primo piano' (($ovs | Where-Object { $_ -eq $fg }).Count -eq 1)
  if (($ovs | Where-Object { $_ -eq $fg }).Count -eq 1) { [MO]::Key(0x1B) }
  Check 'Esc chiude' (Wait-Until { -not (IsOpen) } 4)
  Check 'stato chiuso e miniature liberate' ((Inv 'mc_state').open -eq $false -and (Inv 'mc_state').thumbs -eq 0)
  Start-Sleep -Milliseconds 500
  Check 'la pagina non conserva piu celle ne titoli' ([int](Mc $mc "document.querySelectorAll('.mc-cell').length") -eq 0)

  Write-Host 'E. clic su una cella attiva la finestra'
  [void][MO]::SetForegroundWindow($note); Start-Sleep -Milliseconds 400
  Check 'si riapre' (Open-Mc)
  $mc = Monitor-Of $calcId; $cc = Cell-Info $mc $calcId
  $p = Screen-Point $mc $cc 0.5 0.4
  [void][MO]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 400
  $elem = Mc $mc "(function(){var e=document.elementFromPoint($($p.X - [int](Js 'mc' (McIdx $mc) 'window.screenX')), $($p.Y - [int](Js 'mc' (McIdx $mc) 'window.screenY')));var c=e&&e.closest('.mc-cell');return c?c.dataset.id:null})()"
  Check 'sotto il cursore c e proprio la cella della Calcolatrice di prova (si clicca solo se e vero)' ("$elem" -eq "$calcId")
  if ("$elem" -eq "$calcId") { [MO]::Click() }
  Check 'la panoramica si chiude' (Wait-Until { -not (IsOpen) } 4)
  Check 'la Calcolatrice e in primo piano' (Wait-Until { [MO]::GetForegroundWindow() -eq $calc } 4)

  Write-Host 'F. finestra minimizzata: scheda, un clic la ripristina'
  [void][MO]::ShowWindow($note, 6); Start-Sleep -Milliseconds 800
  Check 'il Blocco note di prova e minimizzato' ([MO]::IsIconic($note))
  Check 'si riapre' (Open-Mc)
  $mn = Monitor-Of $noteId; $cn = Cell-Info $mn $noteId
  Check 'cella del Blocco note: scheda minimizzata' (($cn.card -eq 'true') -and ($cn.min -eq 'true'))
  Check 'la scheda mostra l etichetta Minimizzata' ((Mc $mn "(function(){var e=$(CellJs $noteId);var b=e&&e.querySelector('.mc-card-badge');return b?b.textContent:null})()") -eq 'Minimizzata')
  $p = Screen-Point $mn $cn 0.5 0.4
  [void][MO]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 400
  $sxn = [int](Js 'mc' (McIdx $mn) 'window.screenX'); $syn = [int](Js 'mc' (McIdx $mn) 'window.screenY')
  $elem = Mc $mn "(function(){var e=document.elementFromPoint($($p.X - $sxn), $($p.Y - $syn));var c=e&&e.closest('.mc-cell');return c?c.dataset.id:null})()"
  Check 'sotto il cursore c e la scheda del Blocco note di prova' ("$elem" -eq "$noteId")
  if ("$elem" -eq "$noteId") { [MO]::Click() }
  Check 'il Blocco note viene ripristinato' (Wait-Until { -not [MO]::IsIconic($note) } 4)
  Check 'e portato in primo piano' (Wait-Until { [MO]::GetForegroundWindow() -eq $note } 4)

  Write-Host 'G. pulsante di chiusura di una cella'
  Check 'si riapre' (Open-Mc)
  $mc = Monitor-Of $calcId; $cc = Cell-Info $mc $calcId
  $p = Screen-Point $mc $cc 0.5 0.4
  [void][MO]::SetCursorPos($p.X, $p.Y); Start-Sleep -Milliseconds 400   # evidenzia la cella: il pulsante diventa visibile
  $sx = [int](Js 'mc' (McIdx $mc) 'window.screenX'); $sy = [int](Js 'mc' (McIdx $mc) 'window.screenY')
  $b = Mc $mc "(function(){var e=$(CellJs $calcId);var r=e.querySelector('.mc-close').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $bx = [int]($sx + $b[0]); $by = [int]($sy + $b[1])
  [void][MO]::SetCursorPos($bx, $by); Start-Sleep -Milliseconds 400
  $isBtn = Mc $mc "(function(){var e=document.elementFromPoint($($bx - $sx), $($by - $sy));return e&&e.classList.contains('mc-close')?e.closest('.mc-cell').dataset.id:null})()"
  Check 'sotto il cursore c e il pulsante di chiusura della cella di prova' ("$isBtn" -eq "$calcId")
  $cellsBefore = [int](Mc $mc "document.querySelectorAll('.mc-cell').length")
  if ("$isBtn" -eq "$calcId") { [MO]::Click() }
  Check 'la Calcolatrice di prova si chiude (WM_CLOSE)' (Wait-Until { -not [MO]::IsWindow($calc) -or -not [MO]::IsWindowVisible($calc) } 6)
  Check 'la panoramica resta aperta' (IsOpen)
  Check 'la cella sparisce dalla panoramica' (Wait-Until { [int](Mc $mc "document.querySelectorAll('.mc-cell').length") -eq ($cellsBefore - 1) } 6) ("prima $cellsBefore")
  $mine = @($note)

  Write-Host 'H. clic sullo sfondo e perdita del focus'
  $mm = Monitor-Of $noteId
  $i = McIdx $mm; $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  $ex = Js 'mc' $i "(function(){var w=window.innerWidth,h=window.innerHeight;var pts=[[8,8],[w-8,8],[8,h-8],[w-8,h-8],[w/2,24]];for(var k=0;k<pts.length;k++){var e=document.elementFromPoint(pts[k][0],pts[k][1]);if(e&&!e.closest('.mc-cell'))return pts[k]}return null})()"
  if ($ex) {
    [void][MO]::SetCursorPos([int]($sx + $ex[0]), [int]($sy + $ex[1])); Start-Sleep -Milliseconds 400
    [MO]::Click()
    Check 'un clic sullo sfondo chiude' (Wait-Until { -not (IsOpen) } 4)
  } else { Write-Host '  (nessun punto libero trovato)' -ForegroundColor Yellow }
  Check 'si riapre' (Open-Mc)
  [void][MO]::SetForegroundWindow($note)
  Check 'passare a un altra finestra chiude la panoramica' (Wait-Until { -not (IsOpen) } 4)

  Write-Host 'I. sicurezza: id non validi'
  Check 'con la panoramica chiusa nessun id e valido' ((Inv 'mc_activate' ", {id: $noteId}") -eq $false)
  Check 'neppure per chiudere' ((Inv 'mc_close_window' ", {id: $noteId}") -eq $false)
  Check 'si riapre' (Open-Mc)
  Check 'id non elencato: attivazione rifiutata' ((Inv 'mc_activate' ", {id: 4}") -eq $false)
  Check 'id non elencato: chiusura rifiutata' ((Inv 'mc_close_window' ", {id: 4}") -eq $false)
  Check 'la panoramica e ancora aperta (rifiuti senza effetti)' (IsOpen)
  Esc-If-Open
  [void](Inv 'mc_close'); Start-Sleep -Milliseconds 500

  Write-Host 'J. apertura dal Launcher'
  [void][MO]::SetCursorPos(([System.Windows.Forms.Screen]::PrimaryScreen.Bounds.X + 400), 400)
  [MO]::AltSpace(); Start-Sleep -Milliseconds 1000
  $lw = [MO]::Titled('EosOS Launcher'); $lfocus = ($lw.Count -gt 0 -and [MO]::GetForegroundWindow() -eq $lw[0])
  Check 'Launcher aperto e a fuoco' $lfocus
  if ($lfocus) {
    foreach ($c in 'mission'.ToCharArray()) { [MO]::Key([byte][char]$c.ToString().ToUpper()); Start-Sleep -Milliseconds 70 }
    Start-Sleep -Milliseconds 900
    [MO]::Key(0x0D)
    Check 'Invio su "Mission Control" apre la panoramica' (Wait-Until { IsOpen } 6)
    Esc-If-Open
  }

  Write-Host 'K. fascia dei desktop virtuali (sola lettura: nessun desktop viene creato o cambiato)'
  $reg = 'HKCU:\Software\Microsoft\Windows\CurrentVersion\Explorer\VirtualDesktops'
  $regIds = $null; try { $regIds = (Get-ItemProperty $reg -ErrorAction Stop).VirtualDesktopIDs } catch {}
  $regCount = if ($regIds) { [int]($regIds.Length / 16) } else { 1 }
  $dl = @(Inv 'mc_desktops')
  Check 'mc_desktops coincide con il registro' ($dl.Count -eq $regCount) ("desktop " + $dl.Count)
  Check 'un solo desktop e quello corrente' (@($dl | Where-Object { $_.current }).Count -eq 1)
  Check 'si apre' (Open-Mc)
  $mm = 0; if ((McIdx 0) -lt 0) { $mm = 1 }
  $nChips = [int](Mc $mm "document.querySelectorAll('.mc-space').length")
  Check 'la fascia ha un riquadro per desktop piu il +' ($nChips -eq ($dl.Count + 1)) "riquadri $nChips"
  Check 'esattamente uno e il corrente' ([int](Mc $mm "document.querySelectorAll('.mc-space[data-current=true]').length") -eq 1)
  Check 'il riquadro corrente ha nome e numero' ((Mc $mm "document.querySelector('.mc-space[data-current=true] .mc-space-name').textContent").Length -gt 0)
  $barBottom = [double](Mc $mm "document.querySelector('.mc-spaces').getBoundingClientRect().bottom")
  $cellsTop = [double](Mc $mm "(function(){var t=1e9;document.querySelectorAll('.mc-cell').forEach(function(e){t=Math.min(t,e.getBoundingClientRect().top)});return t})()")
  Check 'la fascia sta sopra le celle, senza sovrapporsi' ($barBottom -le $cellsTop) ("fascia fino a $barBottom, celle da $cellsTop")
  # Clic vero sul riquadro del desktop corrente: chiude la panoramica (non cambia nulla).
  $i = McIdx $mm; $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  $chip = Mc $mm "(function(){var r=document.querySelector('.mc-space[data-current=true]').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $chx = [int]($sx + $chip[0]); $chy = [int]($sy + $chip[1])
  [void][MO]::SetCursorPos($chx, $chy); Start-Sleep -Milliseconds 400
  $isChip = Mc $mm "(function(){var e=document.elementFromPoint($($chx - $sx), $($chy - $sy));var c=e&&e.closest('.mc-space');return c&&c.dataset.current==='true'?'si':'no'})()"
  Check 'sotto il cursore c e il riquadro del desktop corrente (si clicca solo se e vero)' ($isChip -eq 'si')
  if ($isChip -eq 'si') { [MO]::Click() }
  Check 'cliccare il desktop corrente chiude la panoramica' (Wait-Until { -not (IsOpen) } 4)
  Check 'indice non valido: rifiutato' ((Inv 'mc_switch_desktop' ', {index: 99}') -eq $false)
  Check 'desktop gia corrente: rifiutato (nessun tasto parte)' ((Inv 'mc_switch_desktop' ', {index: 0}') -eq $false)
  Check 'nuovo desktop con la panoramica chiusa: rifiutato (nessun tasto parte)' ((Inv 'mc_new_desktop') -eq $false)
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  try { [void](Inv 'mc_close') } catch {}
  foreach ($h in $mine) { if ($h -ne [IntPtr]::Zero -and [MO]::IsWindow($h)) { [void][MO]::PostMessage($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) } }
  Start-Sleep 1
  Cleanup-Eos
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
