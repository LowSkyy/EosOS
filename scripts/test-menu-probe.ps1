# Menu globale, fase G1: misura in SOLA LETTURA di cosa si legge dei menu delle finestre. Due finestre di prova
# con menu noti (una con un menu classico HMENU, una con una barra dei menu di .NET/UI Automation), una console e
# la Calcolatrice di prova NUOVE; poi un riepilogo delle finestre dell'utente PER PROGRAMMA (solo nome del
# programma e conteggi: mai titoli di finestra ne' testi dei menu). Nessuna voce viene invocata, nessun clic, nessun tasto.
#
# Uso: powershell -File scripts\test-menu-probe.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MP {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long[] Visible() { var o = new System.Collections.Generic.List<long>(); EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Length > 0) o.Add(h.ToInt64()); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }

# Due finestre di prova: A con un menu classico (MainMenu = HMENU vero), B con una barra MenuStrip (UI Automation).
$formA = Join-Path $env:TEMP 'eosos-menu-a.ps1'; $formB = Join-Path $env:TEMP 'eosos-menu-b.ps1'
Set-Content $formA @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Menu-A'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MainMenu
foreach ($n in 'File', 'Modifica', 'Visualizza', 'Aiuto') { $i = New-Object System.Windows.Forms.MenuItem $n; [void]$i.MenuItems.Add('Voce'); [void]$m.MenuItems.Add($i) }
$f.Menu = $m; [void]$f.ShowDialog()
'@ -Encoding UTF8
Set-Content $formB @'
Add-Type -AssemblyName System.Windows.Forms
$f = New-Object System.Windows.Forms.Form; $f.Text = 'EosOS-Menu-B'; $f.Width = 500; $f.Height = 300
$m = New-Object System.Windows.Forms.MenuStrip
foreach ($n in 'File', 'Modifica', 'Aiuto') { $i = New-Object System.Windows.Forms.ToolStripMenuItem $n; [void]$i.DropDownItems.Add('Voce'); [void]$m.Items.Add($i) }
$f.MainMenuStrip = $m; $f.Controls.Add($m); [void]$f.ShowDialog()
'@ -Encoding UTF8
try {
  Cleanup-Eos; Start-Sleep 1
  $before = [MP]::Visible()
  Write-Host 'Preparazione: finestre di prova NUOVE'
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formA | Out-Null
  Start-Process powershell -ArgumentList '-NoProfile', '-STA', '-WindowStyle', 'Hidden', '-File', $formB | Out-Null
  Start-Process 'conhost.exe' -ArgumentList 'powershell.exe -NoProfile -NoExit -Command "$Host.UI.RawUI.WindowTitle=''EosOS-Menu-C''"' | Out-Null
  Start-Process 'calc.exe' | Out-Null
  $ids = @{}
  Wait-Until { $ids.A = [MP]::Find('EosOS-Menu-A'); $ids.B = [MP]::Find('EosOS-Menu-B'); $ids.C = [MP]::Find('EosOS-Menu-C'); $ids.D = [MP]::Find('Calcolatrice'); ($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0) -and ($ids.D -ne 0) } 20 | Out-Null
  Check 'finestre di prova create' (($ids.A -ne 0) -and ($ids.B -ne 0) -and ($ids.C -ne 0) -and ($ids.D -ne 0))
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 10
  $t0 = Get-Date
  $rows = @(Js "window.__TAURI_INTERNALS__.invoke('menu_probe_all')" | ForEach-Object { $_ })
  $total = ((Get-Date) - $t0).TotalSeconds
  Write-Host ("  misurate {0} finestre in {1:N1} s" -f $rows.Count, $total)
  function Row($id) { $rows | Where-Object { [int64]$_.id -eq [int64]$id } | Select-Object -First 1 }

  Write-Host 'A. finestre di prova con menu noti'
  $a = Row $ids.A; $b = Row $ids.B; $c = Row $ids.C; $d = Row $ids.D
  Check 'A (menu classico): l HMENU ha 4 voci, tutte con titolo' (($a.probe.hmenu_items -eq 4) -and ($a.probe.hmenu_titled -eq 4)) ("voci " + $a.probe.hmenu_items + ", con titolo " + $a.probe.hmenu_titled)
  Check 'A: classificata come leggibile' (@('hmenu', 'both') -contains $a.readability) ("classe " + $a.readability + ", UIA: barra " + $a.probe.uia_menubar + " voci " + $a.probe.uia_items)
  Check 'B (MenuStrip): nessun HMENU' ($null -eq $b.probe.hmenu_items)
  Check 'B: UI Automation trova la barra con 3 voci' (($b.probe.uia_menubar -eq $true) -and ($b.probe.uia_items -eq 3)) ("barra " + $b.probe.uia_menubar + " voci " + $b.probe.uia_items)
  Check 'B: classificata come leggibile (uia)' ($b.readability -eq 'uia') ("classe " + $b.readability)
  Check 'C (console): nessun menu da portare in alto' ($c.readability -eq 'none') ("classe " + $c.readability + ", barre UIA: " + (@($c.probe.uia_bar_ids) -join ','))
  Check 'D (Calcolatrice moderna): classe misurata e tempi ragionevoli' (($null -ne $d) -and ($d.probe.uia_ms -lt 1500)) ("classe " + $d.readability + ", " + $d.probe.uia_ms + " ms, barre UIA: " + (@($d.probe.uia_bar_ids) -join ','))
  Check 'nessuna finestra ha fatto scadere UI Automation oltre il tempo massimo (altrimenti e Unknown)' (@($rows | Where-Object { $_.probe.uia_timed_out }).Count -eq 0) ("scadute " + @($rows | Where-Object { $_.probe.uia_timed_out }).Count)

  Write-Host 'B. riepilogo PER PROGRAMMA delle finestre aperte (solo nome del programma e conteggi, nessun titolo)'
  $tests = @($ids.A, $ids.B, $ids.C, $ids.D) | ForEach-Object { [int64]$_ }
  $others = @($rows | Where-Object { $tests -notcontains [int64]$_.id })
  $others | Group-Object app | Sort-Object Name | ForEach-Object {
    $g = $_.Group
    $cls = ($g | Group-Object readability | ForEach-Object { $_.Name + '=' + $_.Count }) -join ' '
    $ms = ($g | ForEach-Object { $_.probe.uia_ms } | Measure-Object -Maximum).Maximum
    $bars = (($g | ForEach-Object { @($_.probe.uia_bar_ids) } | Select-Object -Unique) -join ',')
    Write-Host ("    {0,-28} {1}   (UIA max {2} ms, barre: {3})" -f $_.Name, $cls, $ms, $bars)
  }
  $by = $others | Group-Object readability | ForEach-Object { $_.Name + '=' + $_.Count }
  Write-Host ("  Totale finestre dell utente: " + $others.Count + "   " + ($by -join ' '))

  Write-Host 'C. programmi comuni di Windows, aperti NUOVI per la prova e richiusi (WM_CLOSE) alla fine'
  $known = @($rows | ForEach-Object { [int64]$_.id })
  $made = @()
  foreach ($cmd in @(@('notepad.exe'), @('mspaint.exe'), @('explorer.exe', $env:TEMP), @('wordpad.exe'), @('control.exe'), @('cmd.exe'))) {
    try { if ($cmd.Count -gt 1) { Start-Process $cmd[0] -ArgumentList $cmd[1] -ErrorAction Stop | Out-Null } else { Start-Process $cmd[0] -ErrorAction Stop | Out-Null } } catch { Write-Host ("    (non avviabile: " + $cmd[0] + ")") }
  }
  Start-Sleep 6
  $rows2 = @(Js "window.__TAURI_INTERNALS__.invoke('menu_probe_all')" | ForEach-Object { $_ })
  $new = @($rows2 | Where-Object { $known -notcontains [int64]$_.id -and $tests -notcontains [int64]$_.id })
  foreach ($r in $new) {
    Write-Host ("    {0,-26} classe={1,-8} HMENU={2} UIA(voci={3}, {4} ms) barre=[{5}]" -f $r.app, $r.readability, $r.probe.hmenu_items, $r.probe.uia_items, $r.probe.uia_ms, (@($r.probe.uia_bar_ids) -join ','))
    $made += [int64]$r.id
  }
  Check 'tutte le misure sono state fatte nei tempi' (@($new | Where-Object { $_.probe.uia_timed_out }).Count -eq 0)
  foreach ($h in $made) { [void][MP]::PostMessage([IntPtr]$h, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Start-Sleep 2
}
finally {
  Cleanup-Eos
  Get-CimInstance Win32_Process | Where-Object { $_.CommandLine -like '*eosos-menu-*' -or $_.CommandLine -like '*EosOS-Menu-C*' } | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
  if ($ids.D) { [void][MP]::PostMessage([IntPtr]$ids.D, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Remove-Item $formA, $formB -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
