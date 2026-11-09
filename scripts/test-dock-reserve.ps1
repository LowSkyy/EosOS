# Verifica la riserva di spazio del Dock (appbar in basso): le finestre massimizzate si
# fermano sopra la capsula. Non fa clic e non tocca la taskbar.
#
# Uso: powershell -File scripts\test-dock-reserve.ps1 [-Exe percorso\eosos.exe]
#
# Scenari:
#   A. con la riserva: l'area di lavoro si riduce di reserve_height e un Blocco note
#      massimizzato si ferma sopra la capsula del Dock.
#   B. EOSOS_DOCK_RESERVE=0: nessuna riduzione.
#   C. auto-nascondi attivo (dock.json): nessuna riduzione.
#   D. uscita normale: l'area di lavoro torna com'era.
#   E. kill brutale: l'area di lavoro torna com'era (il sistema rimuove l'appbar).
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = Resolve-Path $Exe } else { $exe = Join-Path $PSScriptRoot '..\src-tauri\target\debug\eosos.exe' | Resolve-Path }
# Le barre in alto non c'entrano: l'area di lavoro "di partenza" e' misurata a EosOS spento.
Remove-Item env:EOSOS_DOCK_RESERVE -ErrorAction SilentlyContinue
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class DR {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, ref R r, uint c);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static R Work() { R r = new R(); SystemParametersInfo(0x30, 0, ref r, 0); return r; }
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  public static System.Collections.Generic.List<string> Of(uint pid) { var o = new System.Collections.Generic.List<string>(); EnumWindows((h, l) => { uint p; GetWindowThreadProcessId(h, out p); if (p == pid) { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); R r; GetWindowRect(h, out r); o.Add(h + " vis=" + IsWindowVisible(h) + " '" + sb + "' (" + r.L + "," + r.T + "," + r.Rt + "," + r.B + ")"); } return true; }, IntPtr.Zero); return o; }
  public static IntPtr Find(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$dockJson = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$backup = $null
if (Test-Path $dockJson) { $backup = Get-Content $dockJson -Raw }
function Cleanup { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
# Chiude solo i Blocco note nati dopo quelli gia' presenti all'avvio del test. L'elenco di quelli
# gia' presenti si registra QUI, prima di tutto il resto: se il test fallisce presto, il blocco
# `finally` non deve mai trovare un elenco vuoto e chiudere i Blocco note dell'utente (con testo
# non salvato).
$script:notepadBefore = @(Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
function Stop-NewNotepad {
  if ($null -eq $script:notepadBefore) { return }   # senza elenco di riferimento non si chiude nulla
  Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $script:notepadBefore -notcontains $_.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
}
function Restore-Json { if ($null -ne $backup) { [IO.File]::WriteAllText($dockJson, $backup, (New-Object Text.UTF8Encoding($false))) } }
function Set-Autohide($on) {
  $j = Get-Content $dockJson -Raw | ConvertFrom-Json
  if ($j.PSObject.Properties.Name -contains 'autohide') { $j.autohide = $on } else { $j | Add-Member autohide $on }
  [IO.File]::WriteAllText($dockJson, ($j | ConvertTo-Json -Depth 5), (New-Object Text.UTF8Encoding($false)))
}
$fail = 0
function Check($l, $c) { if ($c) { Write-Host "[OK]    $l" } else { Write-Host "[FALLITO] $l" -ForegroundColor Red; $script:fail++ } }
function Start-Eos { $p = Start-Process $exe -PassThru; Start-Sleep 9; $p }
# Attende (al massimo 20 s) che la finestra del Dock sia visibile e ne misura il tempo.
function Wait-Dock {
  $t0 = Get-Date
  while (((Get-Date) - $t0).TotalSeconds -lt 20) { $h = [DR]::Find('EosOS Dock'); if ($h -ne [IntPtr]::Zero) { Write-Host ("Dock visibile dopo {0:N1} s dall'avvio + 9 s" -f ((Get-Date) - $t0).TotalSeconds); return $h }; Start-Sleep -Milliseconds 300 }
  [IntPtr]::Zero
}
# Le impostazioni del Dock dell'utente (nascondi automaticamente...) falserebbero le misure: parte da quelle di base e il suo file si ripristina.
$isoDir = Join-Path $env:LOCALAPPDATA 'EosOS'
$isoBackup = @{}
foreach ($isoF in 'dock.json', 'mission.json') { $isoP = Join-Path $isoDir $isoF; $isoBackup[$isoF] = if (Test-Path $isoP) { [System.IO.File]::ReadAllBytes($isoP) } else { $null } }
try {
  foreach ($isoF in 'dock.json', 'mission.json') { Remove-Item (Join-Path $isoDir $isoF) -ErrorAction SilentlyContinue }
  Cleanup; Start-Sleep 1
  $before = [DR]::Work(); Write-Host "area di lavoro a EosOS spento: $($before.T)..$($before.B)"

  Write-Host "`n=== A. con la riserva ==="
  $p = Start-Eos
  $a = [DR]::Work(); Write-Host "area di lavoro: $($a.T)..$($a.B)"
  $dock = Wait-Dock; Check "finestra del Dock trovata" ($dock -ne [IntPtr]::Zero)
  if ($dock -eq [IntPtr]::Zero) {
    Write-Host 'DIAGNOSI: Dock non trovato.' -ForegroundColor Yellow
    Get-Process eosos -ErrorAction SilentlyContinue | ForEach-Object { Write-Host "  eosos pid $($_.Id) avviato $($_.StartTime.ToString('HH:mm:ss'))"; [DR]::Of([uint32]$_.Id) | ForEach-Object { Write-Host "    $_" } }
    Write-Host "  processi WebView2: $(@(Get-Process msedgewebview2 -ErrorAction SilentlyContinue).Count)"
    Write-Host "  dock.json: $((Get-Content $dockJson -Raw) -replace '\s+',' ' | Select-Object -First 1)"
    throw 'Dock non trovato: mi fermo'
  }
  $dr = New-Object DR+R; [void][DR]::GetWindowRect($dock, [ref]$dr)
  $reduced = $before.B - $a.B
  # 92 px a scala 100%: capsula 78 + margine 8 + respiro 6 (vedi dockgeom::reserve_height)
  Check "il fondo dell'area di lavoro sale (di $reduced px, atteso > 60)" ($reduced -gt 60)
  $capTop = $dr.T + 90
  Check "il Dock sta appoggiato in basso e non si sposta verso l'alto (fondo finestra = fondo schermo - margine)" ([Math]::Abs($dr.B - ($before.B - 8)) -le 2)
  # Una finestra MIA massimizzata (nessun clic: la si massimizza via API). Prima si usava il Blocco note, ma quello di
  # Windows 11 parte da un lanciatore e ripristina uno stato proprio: il controllo falliva anche senza il Dock.
  $form = Start-Process powershell -ArgumentList '-NoProfile', '-WindowStyle', 'Hidden', '-Command', "Add-Type -AssemblyName System.Windows.Forms; `$f = New-Object System.Windows.Forms.Form; `$f.Text = 'EOSOS-RESERVE-TEST'; `$f.Width = 500; `$f.Height = 300; [System.Windows.Forms.Application]::Run(`$f)" -PassThru -WindowStyle Hidden
  Start-Sleep 5
  $h = [IntPtr]::Zero
  try { $h = (Get-Process -Id $form.Id -ErrorAction Stop).MainWindowHandle } catch {}
  if ($h -eq [IntPtr]::Zero) { Write-Host "[SALTATO] la finestra di prova non si e aperta" -ForegroundColor Yellow }
  if ($h -ne [IntPtr]::Zero) {
    [void][DR]::ShowWindow($h, 3); Start-Sleep 1.5
    $nr = New-Object DR+R; [void][DR]::GetWindowRect($h, [ref]$nr)
    Write-Host "Finestra di prova massimizzata: $($nr.T)..$($nr.B)   capsula del Dock: da $capTop"
    # Una finestra massimizzata ha ~8 px di cornice invisibile oltre l'area di lavoro.
    Check "la finestra massimizzata finisce sopra la capsula (fondo <= $($capTop + 8 + 2))" ($nr.B -le ($capTop + 8 + 2))
    Check "e arriva comunque fin dove consentito (fondo >= area di lavoro)" ($nr.B -ge $a.B)
  }
  if ($form) { Stop-Process -Id $form.Id -Force -ErrorAction SilentlyContinue }
  Stop-NewNotepad

  Write-Host "`n=== D. uscita normale ==="
  # Chiusura gentile (WM_CLOSE) di una finestra di EosOS: l'app deve uscire da sola e ripristinare
  # tutto. Niente ripiego sul kill: dopo un kill e' il sistema a togliere l'appbar, e il controllo
  # passerebbe anche se il percorso di uscita normale fosse rotto.
  $bar = [DR]::Find('EosOS')
  Check "finestra di EosOS trovata per la chiusura gentile" ($bar -ne [IntPtr]::Zero)
  if ($bar -ne [IntPtr]::Zero) { [void][DR]::PostMessage($bar, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  $t0 = Get-Date; while (-not $p.HasExited -and ((Get-Date) - $t0).TotalSeconds -lt 12) { Start-Sleep -Milliseconds 300 }
  Check "l'uscita normale termina EosOS (senza kill)" ($p.HasExited)
  Start-Sleep 2
  $d = [DR]::Work(); Write-Host "area di lavoro: $($d.T)..$($d.B)"
  Check "dopo l'uscita normale l'area di lavoro e' come prima" ($d.T -eq $before.T -and $d.B -eq $before.B)
  if (-not $p.HasExited) { Cleanup; Start-Sleep 3 }   # solo per non lasciare processi, dopo i controlli

  Write-Host "`n=== E. kill brutale ==="
  $p = Start-Eos
  Check "riserva di nuovo attiva" (([DR]::Work()).B -lt $before.B)
  Cleanup; Start-Sleep 6
  $e = [DR]::Work(); Write-Host "area di lavoro: $($e.T)..$($e.B)"
  Check "dopo il kill l'area di lavoro e' come prima" ($e.T -eq $before.T -and $e.B -eq $before.B)

  Write-Host "`n=== B. EOSOS_DOCK_RESERVE=0 ==="
  $env:EOSOS_DOCK_RESERVE = '0'; $p = Start-Eos; Remove-Item env:EOSOS_DOCK_RESERVE
  $b = [DR]::Work(); Write-Host "area di lavoro: $($b.T)..$($b.B)"
  Check "nessuna riduzione in basso" ($b.B -eq $before.B)
  Cleanup; Start-Sleep 4

  Write-Host "`n=== C. auto-nascondi attivo ==="
  if ($null -eq $backup) { Write-Host "(dock.json assente: scenario saltato)" } else {
    Set-Autohide $true; $p = Start-Eos
    $c = [DR]::Work(); Write-Host "area di lavoro: $($c.T)..$($c.B)"
    Check "con l'auto-nascondi non si riserva spazio" ($c.B -eq $before.B)
    Cleanup; Start-Sleep 4
  }
}
finally {
  foreach ($isoF in 'dock.json', 'mission.json') { $isoP = Join-Path $isoDir $isoF; Remove-Item $isoP -ErrorAction SilentlyContinue; if ($null -ne $isoBackup[$isoF]) { [System.IO.File]::WriteAllBytes($isoP, $isoBackup[$isoF]) } }
  Cleanup
  Stop-NewNotepad
  Restore-Json
  Start-Sleep 2
  $end = [DR]::Work(); Write-Host "`narea di lavoro finale: $($end.T)..$($end.B)"
}
if ($fail -gt 0) { Write-Host "`nFALLITI: $fail" -ForegroundColor Red; exit 1 } else { Write-Host "`nTutto OK" }
