# Verifica M5 di Mission Control (desktop virtuali, backend interno con profili e degradazione) sul sistema
# vero, SENZA modificare nulla ai desktop dell'utente: niente creazione, rimozione, rinomina, spostamento o
# passaggio di desktop. Le operazioni che modificano (e quindi gli indici dei metodi relativi) si provano solo
# con scripts\test-vdesk-internal.ps1 e l'ok esplicito dell'utente.
#
# Bersagli di tutti i clic e del mouse premuto: una Calcolatrice di prova NUOVA, dopo aver verificato che sia
# l'elemento sotto il cursore. Le finestre dell'utente compaiono nella panoramica ma non si toccano e i loro
# titoli non si stampano. File dell'utente (vdesk-profiles.json, vdesk-crash.flag) salvati e ripristinati.
#
# Uso: powershell -File scripts\test-mc-vdesk.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class VT {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr[] Titled(string t) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { if (Title(h) == t && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Down() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); }
  public static void Up() { mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$i, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Inv([string]$cmd, [string]$extra = '') { Js 'launcher' 0 "window.__TAURI_INTERNALS__.invoke('$cmd'$extra)" }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Overlays { @([VT]::Titled('EosOS Mission Control')) }
function IsOpen { (Overlays).Count -gt 0 }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 11 }
function Open-Mc { [void](Inv 'mc_open'); Wait-Until { (Inv 'mc_state').phase -eq 'open' } 8 }
function Close-Mc { [void](Inv 'mc_close'); Wait-Until { (Inv 'mc_state').phase -eq 'closed' } 4 | Out-Null; Start-Sleep -Milliseconds 300 }
function McIdx([int]$m) { foreach ($i in 0..4) { $v = Js 'mc' $i "new URLSearchParams(location.search).get('m')"; if ($null -ne $v -and [int]$v -eq $m) { return $i } }; -1 }
function Mc([int]$m, [string]$expr) { $i = McIdx $m; if ($i -ge 0) { Js 'mc' $i $expr } }
function CellJs($id) { "Array.from(document.querySelectorAll('.mc-cell')).find(function(e){return e.dataset.id==='$id'})" }
function Find-New($before, [string]$namePart) { foreach ($h in [VT]::Visible()) { if ($before -notcontains $h -and ([VT]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function Monitor-Of([long]$id) { foreach ($m in 0..3) { if ((McIdx $m) -ge 0) { $c = Mc $m "(function(){var e=$(CellJs $id);return e?1:null})()"; if ($c) { return $m } } }; -1 }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$flag = Join-Path $dir 'vdesk-crash.flag'; $profs = Join-Path $dir 'vdesk-profiles.json'; $rep = Join-Path $dir 'vdesk-report.json'
$saved = @{}; foreach ($f in $flag, $profs) { if (Test-Path $f) { $saved[$f] = Get-Content $f -Raw; Remove-Item $f } }
$calc = [IntPtr]::Zero
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [VT]::Visible()
  Start-Process 'calc.exe' | Out-Null
  $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 15 -and $calc -eq [IntPtr]::Zero) { Start-Sleep -Milliseconds 400; $calc = Find-New $before 'Calcolatrice' }
  if ($calc -eq [IntPtr]::Zero) { throw 'Calcolatrice di prova non trovata' }
  $calcId = $calc.ToInt64()
  Start-Eos

  Write-Host 'A. rapporto diagnostico (nessun dato dell utente)'
  $r = Inv 'mc_vd_report' ', {refresh: true}'
  $k = Get-ItemProperty 'HKLM:\SOFTWARE\Microsoft\Windows NT\CurrentVersion'
  Check 'la build nel rapporto e quella di Windows' (($r.build -eq [int]$k.CurrentBuild) -and ($r.revision -eq [int]$k.UBR)) ("build " + $r.build + "." + $r.revision + " " + $r.display_version)
  Check 'un profilo ha superato la verifica' ($r.selected -eq 'w11-24h2-and-later') ("selezionato: " + $r.selected)
  Check 'tutti i controlli del tentativo sono riusciti' ((@($r.attempts[0].checks | Where-Object { -not $_.ok })).Count -eq 0) ("controlli: " + @($r.attempts[0].checks).Count)
  Check 'il profilo dichiara la build' ($r.attempts[0].declared -eq $true)
  Check 'capacita interne attive' (($r.caps.internal -eq $true) -and ($r.caps.move_window -eq $true) -and ($r.caps.remove_any -eq $true) -and ($r.caps.rename -eq $true))
  $txt = ($r | ConvertTo-Json -Depth 8)
  Check 'nessun percorso o nome dell utente nel rapporto' (($txt -notmatch [regex]::Escape($env:USERNAME)) -and ($txt -notmatch '[A-Z]:\\'))
  Check 'dopo il tentativo il segnale di crash non resta' (-not (Test-Path $flag))

  Write-Host 'B. panoramica con il backend interno'
  Check 'si apre' (Open-Mc)
  $mc = Monitor-Of $calcId
  Check 'la Calcolatrice di prova e in una panoramica' ($mc -ge 0)
  Check 'nessun avviso sulle funzioni avanzate' ((Mc $mc "document.querySelectorAll('.mc-hint').length") -eq 0)
  Check 'con un solo desktop non c e il pulsante di rimozione' ((Mc $mc "document.querySelectorAll('.mc-space-remove').length") -eq 0)
  Check 'i comandi si rifiutano se la richiesta non ha senso (desktop corrente)' (((Inv 'mc_move_window' ", {id: $calcId, index: 0}") -eq $false) -and ((Inv 'mc_remove_desktop' ', {index: 0}') -eq $false))
  Check 'id non elencato: spostamento rifiutato' ((Inv 'mc_move_window' ', {id: 4, index: 0}') -eq $false)
  Check 'indice fuori elenco: rifiutato' ((Inv 'mc_move_window' ", {id: $calcId, index: 99}") -eq $false)

  Write-Host 'C. trascinare una cella: etichetta che segue il cursore, nessun clic, nessuna modifica'
  $i = McIdx $mc; $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  $cell = Mc $mc "(function(){var r=$(CellJs $calcId).getBoundingClientRect();return [r.left+r.width/2, r.top+r.height*0.4]})()"
  $chip = Mc $mc "(function(){var r=document.querySelector('.mc-space[data-index=\`"0\`"]').getBoundingClientRect();return [r.left+r.width/2, r.top+r.height/2]})()"
  $cx = [int]($sx + $cell[0]); $cy = [int]($sy + $cell[1]); $hx = [int]($sx + $chip[0]); $hy = [int]($sy + $chip[1])
  [void][VT]::SetCursorPos($cx, $cy); Start-Sleep -Milliseconds 400
  $under = Mc $mc "(function(){var e=document.elementFromPoint($($cx - $sx), $($cy - $sy));var c=e&&e.closest('.mc-cell');return c?c.dataset.id:null})()"
  Check 'sotto il cursore c e la cella della Calcolatrice di prova (si preme il mouse solo se e vero)' ("$under" -eq "$calcId")
  if ("$under" -eq "$calcId") {
    [VT]::Down()
    foreach ($s in 1..8) { [void][VT]::SetCursorPos([int]($cx + ($hx - $cx) * $s / 8), [int]($cy + ($hy - $cy) * $s / 8)); Start-Sleep -Milliseconds 60 }
    Start-Sleep -Milliseconds 300
    Check 'durante il trascinamento compare l etichetta' ((Mc $mc "document.querySelectorAll('.mc-ghost').length") -eq 1)
    Check 'il riquadro sotto il cursore e evidenziato come destinazione' ((Mc $mc "document.querySelectorAll('.mc-space[data-drop=true]').length") -eq 1)
    Check 'la cella trascinata e attenuata' ((Mc $mc "$(CellJs $calcId).dataset.dragging") -eq 'true')
    [VT]::Up(); Start-Sleep -Milliseconds 800
    Check 'al rilascio l etichetta sparisce' ((Mc $mc "document.querySelectorAll('.mc-ghost').length") -eq 0)
    Check 'la cella torna normale' ((Mc $mc "$(CellJs $calcId).dataset.dragging") -eq 'false')
    Check 'la panoramica resta aperta: il rilascio non conta come clic' (IsOpen)
    Check 'nessuno spostamento: il desktop e quello corrente, la finestra e ancora elencata' ((Mc $mc "(function(){return $(CellJs $calcId)?1:0})()") -eq 1)
    Check 'e la finestra di prova non e stata portata in primo piano' ([VT]::GetForegroundWindow() -ne $calc)
  }
  Close-Mc

  Write-Host 'D. profili dell utente: uno rotto e uno con identificatori sbagliati'
  # Profilo con IID sbagliati (valido come file) messo PRIMA, con id diverso, che dichiara la build: deve fallire e
  # lasciare il posto a quello incorporato. Poi un profilo con un GUID non valido: scartato, con il messaggio.
  $good = '{"id":"utente-iid-sbagliati","description":"prova","minBuild":26000,"maxBuild":999999,"manager":{"service":"{C5E0CDCA-7B6E-41B2-9FC4-D93975CC467B}","iid":"{11111111-2222-3333-4444-555555555555}","methods":{"getCount":3,"moveView":4,"canMoveView":5,"getCurrent":6,"getDesktops":7,"getAdjacent":8,"switchTo":9,"create":11,"remove":13,"find":14,"includeExclude":15,"setName":16}},"desktop":{"iid":"{3F07F4BE-B107-441A-AF0F-39D82529072C}","methods":{"getId":4}},"views":{"service":"{1841C6D7-4F9D-42C0-AF41-8747538F10E5}","iid":"{1841C6D7-4F9D-42C0-AF41-8747538F10E5}","methods":{"getViewForHwnd":6}}}'
  $bad = $good.Replace('utente-iid-sbagliati', 'utente-rotto').Replace('{11111111-2222-3333-4444-555555555555}', 'non-un-guid')
  [System.IO.File]::WriteAllText($profs, '{"version":1,"profiles":[' + $good + ',' + $bad + ']}', (New-Object System.Text.UTF8Encoding($false)))
  $r = Inv 'mc_vd_report' ', {refresh: true}'
  Check 'il file dell utente e letto' ($r.user_profiles -eq 'ok')
  Check 'il profilo rotto e scartato con un messaggio' ((@($r.problems | Where-Object { $_ -like '*utente-rotto*' })).Count -eq 1)
  Check 'il profilo con identificatori sbagliati e provato per primo e NON supera la verifica' (($r.attempts[0].profile -eq 'utente-iid-sbagliati') -and ($r.attempts[0].passed -eq $false))
  Check 'il motivo del fallimento e nel rapporto' (@($r.attempts[0].checks | Where-Object { -not $_.ok }).Count -ge 1)
  Check 'si ripiega sul profilo incorporato, che funziona' (($r.selected -eq 'w11-24h2-and-later') -and ($r.caps.internal -eq $true))
  [System.IO.File]::WriteAllText($profs, 'questo non e json {', (New-Object System.Text.UTF8Encoding($false)))
  $r = Inv 'mc_vd_report' ', {refresh: true}'
  Check 'un file dei profili illeggibile non rompe nulla: resta il profilo incorporato' (($r.user_profiles -like '*non leggibile*') -and ($r.selected -eq 'w11-24h2-and-later'))
  Remove-Item $profs

  Write-Host 'E. modalita degradata: tentativo precedente finito male (segnale di crash)'
  Cleanup-Eos; Start-Sleep 2
  [System.IO.File]::WriteAllText($flag, 'prova', (New-Object System.Text.UTF8Encoding($false)))
  Start-Eos
  $r = Inv 'mc_vd_report'
  Check 'il backend interno resta spento' (($r.selected -eq $null) -and ($r.caps.internal -eq $false))
  Check 'il motivo e il crash precedente' ($r.reason -like '*vdesk-crash.flag*')
  Check 'non ha provato nessun profilo (nessuna chiamata sulla tabella virtuale)' (@($r.attempts).Count -eq 0)
  Check 'il segnale non viene cancellato da solo' (Test-Path $flag)
  Check 'si apre lo stesso: elenco, passaggio e creazione restano (backend documentato)' (Open-Mc)
  $mc = Monitor-Of $calcId
  Check 'la fascia dei desktop c e' ((Mc $mc "document.querySelectorAll('.mc-space').length") -ge 2)
  Check 'compare l avviso sulle funzioni avanzate non disponibili' ((Mc $mc "document.querySelectorAll('.mc-hint').length") -eq 1)
  Check 'lo spostamento e rifiutato' ((Inv 'mc_move_window' ", {id: $calcId, index: 0}") -eq $false)
  Check 'la rinomina e rifiutata' ((Inv 'mc_rename_desktop' ', {index: 0, name: ''x''}') -eq $false)
  # Trascinare non fa partire nulla (spostamento non disponibile): nessuna etichetta.
  $i = McIdx $mc; $sx = [int](Js 'mc' $i 'window.screenX'); $sy = [int](Js 'mc' $i 'window.screenY')
  $cell = Mc $mc "(function(){var r=$(CellJs $calcId).getBoundingClientRect();return [r.left+r.width/2, r.top+r.height*0.4]})()"
  $cx = [int]($sx + $cell[0]); $cy = [int]($sy + $cell[1])
  [void][VT]::SetCursorPos($cx, $cy); Start-Sleep -Milliseconds 400
  $under = Mc $mc "(function(){var e=document.elementFromPoint($($cx - $sx), $($cy - $sy));var c=e&&e.closest('.mc-cell');return c?c.dataset.id:null})()"
  if ("$under" -eq "$calcId") {
    [VT]::Down(); foreach ($s in 1..6) { [void][VT]::SetCursorPos($cx + 12 * $s, $cy); Start-Sleep -Milliseconds 50 }
    Check 'senza spostamento disponibile il trascinamento non mostra l etichetta' ((Mc $mc "document.querySelectorAll('.mc-ghost').length") -eq 0)
    [VT]::Up(); Start-Sleep -Milliseconds 600
  }
  Close-Mc
  Cleanup-Eos; Start-Sleep 2
  Remove-Item $flag
  Start-Eos
  $r = Inv 'mc_vd_report'
  Check 'tolto il segnale, al riavvio il backend interno torna attivo' (($r.selected -eq 'w11-24h2-and-later') -and ($r.caps.internal -eq $true))
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  try { [void](Inv 'mc_close') } catch {}
  if ($calc -ne [IntPtr]::Zero -and [VT]::IsWindow($calc)) { [void][VT]::PostMessage($calc, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Start-Sleep 1
  Cleanup-Eos
  foreach ($f in $flag, $profs) { Remove-Item $f -ErrorAction SilentlyContinue; if ($saved.ContainsKey($f)) { [System.IO.File]::WriteAllText($f, $saved[$f], (New-Object System.Text.UTF8Encoding($false))) } }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
