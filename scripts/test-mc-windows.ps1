# Verifica l'elenco delle finestre di Mission Control (M1) sul sistema vero, in SOLA LETTURA: il comando
# `mc_windows` non muove ne attiva nulla. Si creano due finestre di prova (un Blocco note e una
# Calcolatrice NUOVI) e si controlla che compaiano con i dati giusti; quelle dell'utente non si toccano e
# i loro titoli non si stampano (solo quante sono). Le due finestre di prova si chiudono con garbo alla fine.
#
# Uso: powershell -File scripts\test-mc-windows.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MW {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int cmd);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  public static uint Pid(long h) { uint p; GetWindowThreadProcessId(new IntPtr(h), out p); return p; }
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Mc { $env:CDP_VIEW = 'launcher'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval "window.__TAURI_INTERNALS__.invoke('mc_windows').then(function(r){return JSON.stringify(r)})" 2>$null; if ($o) { $arr = ($o | ConvertFrom-Json) | ConvertFrom-Json; foreach ($x in $arr) { $x } } } finally { $ErrorActionPreference = $old } }
function All-Windows { @(Mc | ForEach-Object { $_.windows }) }
function Find-New($before, [string]$namePart) { foreach ($h in [MW]::Visible()) { if ($before -notcontains $h -and ([MW]::Title([IntPtr]$h) -like "*$namePart*")) { return [IntPtr]$h } }; [IntPtr]::Zero }
function Wait-Until-Gone { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt 6) { if (-not (All-Windows | Where-Object { $_.id -eq $calc.ToInt64() })) { return $true }; Start-Sleep -Milliseconds 300 }; $false }
$mine = @()
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [MW]::Visible()
  Write-Host 'Preparazione: una Calcolatrice e un Blocco note NUOVI (le finestre dell utente non si toccano)'
  Start-Process 'calc.exe' | Out-Null
  Start-Process 'notepad.exe' | Out-Null
  $calc = [IntPtr]::Zero; $note = [IntPtr]::Zero
  $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 15 -and ($calc -eq [IntPtr]::Zero -or $note -eq [IntPtr]::Zero)) {
    Start-Sleep -Milliseconds 400
    if ($calc -eq [IntPtr]::Zero) { $calc = Find-New $before 'Calcolatrice' }
    if ($note -eq [IntPtr]::Zero) { $note = Find-New $before 'Blocco note' }
  }
  Check 'finestra di prova Calcolatrice creata' ($calc -ne [IntPtr]::Zero)
  Check 'finestra di prova Blocco note creata' ($note -ne [IntPtr]::Zero)
  if ($calc -eq [IntPtr]::Zero -or $note -eq [IntPtr]::Zero) { throw 'finestre di prova non trovate' }
  $mine = @($calc, $note)
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 9

  Write-Host 'A. elenco e dati'
  $mons = @(Mc)
  Check 'un elemento per monitor' ($mons.Count -ge 1) ("monitor: " + $mons.Count)
  $all = All-Windows
  Write-Host "  finestre elencate in tutto: $($all.Count) (titoli dell utente non stampati)"
  $eosPids = @(Get-Process eosos -ErrorAction SilentlyContinue | ForEach-Object { [uint32]$_.Id })
  Check 'nessuna finestra di EosOS nell elenco (controllo per processo)' (@($all | Where-Object { $eosPids -contains [MW]::Pid($_.id) }).Count -eq 0)
  $wc = $all | Where-Object { $_.id -eq $calc.ToInt64() }
  $wn = $all | Where-Object { $_.id -eq $note.ToInt64() }
  Check 'la Calcolatrice di prova e elencata' ($null -ne $wc)
  Check 'il Blocco note di prova e elencato' ($null -ne $wn)
  Check 'titolo e app della Calcolatrice' (($wc.title -like '*Calcolatrice*') -and ($wc.app.Length -gt 0) -and (-not $wc.minimized)) ("app: " + $wc.app)
  $r = New-Object MW+R; [void][MW]::GetWindowRect($calc, [ref]$r)
  Check 'rettangolo reale della Calcolatrice' (($wc.rect[0] -eq $r.L) -and ($wc.rect[1] -eq $r.T) -and ($wc.rect[2] -eq ($r.Rt - $r.L)) -and ($wc.rect[3] -eq ($r.B - $r.T)))
  Check 'id univoci' (@($all | Select-Object -ExpandProperty id -Unique).Count -eq $all.Count)

  Write-Host 'B. disposizione'
  $okInside = $true; $okOverlap = $true
  foreach ($m in $mons) {
    $k = [double]$m.scale
    $ax = $m.x + [Math]::Round(60 * $k); $ay = $m.y + [Math]::Round(140 * $k)
    $aw = $m.w - 2 * [Math]::Round(60 * $k); $ah = $m.h - [Math]::Round(140 * $k) - [Math]::Round(60 * $k)
    $ws = @($m.windows)
    foreach ($w in $ws) { $c = $w.cell; if ($c[0] -lt $ax -or $c[1] -lt $ay -or ($c[0] + $c[2]) -gt ($ax + $aw) -or ($c[1] + $c[3]) -gt ($ay + $ah)) { $okInside = $false } }
    for ($i = 0; $i -lt $ws.Count; $i++) { for ($j = $i + 1; $j -lt $ws.Count; $j++) { $a = $ws[$i].cell; $b = $ws[$j].cell
      if ($a[0] -lt ($b[0] + $b[2]) -and $b[0] -lt ($a[0] + $a[2]) -and $a[1] -lt ($b[1] + $b[3]) -and $b[1] -lt ($a[1] + $a[3])) { $okOverlap = $false } } }
  }
  Check 'tutte le celle dentro l area del loro monitor (margini e fascia dei desktop esclusi)' $okInside
  Check 'nessuna cella si sovrappone a un altra' $okOverlap
  $ratioOk = $true
  foreach ($w in $all) { if (-not $w.minimized) { $a = $w.rect[2] / [double]$w.rect[3]; $b = $w.cell[2] / [double]$w.cell[3]; if ([Math]::Abs($a - $b) / $a -gt 0.05) { $ratioOk = $false } } }
  Check 'le celle mantengono le proporzioni delle finestre' $ratioOk

  Write-Host 'C. finestra minimizzata = scheda fissa'
  [void][MW]::ShowWindow($note, 6)   # SW_MINIMIZE, solo sulla NOSTRA finestra di prova
  Start-Sleep -Seconds 1
  $wn = All-Windows | Where-Object { $_.id -eq $note.ToInt64() }
  Check 'e ancora elencata ed e marcata minimizzata' (($null -ne $wn) -and $wn.minimized)
  $mon = (Mc) | Where-Object { $_.windows | Where-Object { $_.id -eq $note.ToInt64() } } | Select-Object -First 1
  $k = [double]$mon.scale
  Check "scheda di dimensione fissa (320x200 CSS x scala $k)" (([Math]::Abs($wn.cell[2] - [Math]::Round(320 * $k)) -le 2) -and ([Math]::Abs($wn.cell[3] - [Math]::Round(200 * $k)) -le 2) -or ($wn.cell[2] -lt [Math]::Round(320 * $k))) ("cella " + ($wn.cell -join 'x'))
  [void][MW]::ShowWindow($note, 9)   # SW_RESTORE
  Start-Sleep -Seconds 1
  $wn = All-Windows | Where-Object { $_.id -eq $note.ToInt64() }
  Check 'ripristinata: torna una finestra normale con il suo rettangolo' (($null -ne $wn) -and (-not $wn.minimized) -and ($wn.rect[2] -gt 100))

  Write-Host 'D. una finestra che si chiude sparisce'
  [void][MW]::PostMessage($calc, 0x10, [IntPtr]::Zero, [IntPtr]::Zero)   # WM_CLOSE alla NOSTRA Calcolatrice
  Check 'la Calcolatrice chiusa non e piu elencata' (Wait-Until-Gone)
}
catch { Write-Host "ERRORE: $_" -ForegroundColor Red; $script:fail++ }
finally {
  foreach ($h in $mine) { if ([MW]::IsWindowVisible($h)) { [void][MW]::PostMessage($h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) } }
  Start-Sleep 1
  Cleanup-Eos
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }

