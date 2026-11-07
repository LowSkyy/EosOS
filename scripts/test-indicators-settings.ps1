# Verifica le impostazioni degli indicatori (I4): il menu del clic destro sulla barra decide quali
# indicatori si vedono e se la batteria mostra la percentuale. Stato SIMULATO (build di prova): non
# tocca batteria, rete o volume. Clic veri, ma solo dopo aver verificato che sotto il cursore ci sia
# la barra di EosOS o il menu stesso. Il file indicators.json dell'utente viene salvato e ripristinato.
#
# Uso: powershell -File scripts\test-indicators-settings.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$bytes = [System.IO.File]::ReadAllBytes($exe)
if (-not [System.Text.Encoding]::GetEncoding(28591).GetString($bytes).Contains('EOSOS_FAKE_INDICATORS')) { Write-Host 'Serve una build di prova (feature test-hooks).'; exit 2 }
Add-Type -AssemblyName System.Windows.Forms, UIAutomationClient, UIAutomationTypes
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class IS {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern IntPtr WindowFromPoint(P p);
  [DllImport("user32.dll")] public static extern IntPtr GetAncestor(IntPtr h, uint f);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
  public static IntPtr[] Titled(string title) { var o = new System.Collections.Generic.List<IntPtr>(); EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title && IsWindowVisible(h)) o.Add(h); return true; }, IntPtr.Zero); return o.ToArray(); }
  public static IntPtr Root(int x, int y) { return GetAncestor(WindowFromPoint(new P { x = x, y = y }), 2); }
  public static void Click() { mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static void Esc() { keybd_event(0x1B, 0, 0, UIntPtr.Zero); keybd_event(0x1B, 0, 2, UIntPtr.Zero); }
  public static void RightClick() { mouse_event(8, 0, 0, 0, UIntPtr.Zero); mouse_event(16, 0, 0, 0, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function PJs([int]$i, [string]$expr) { $env:CDP_VIEW = 'bar'; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { $o | ConvertFrom-Json } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 6) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 100 }; $false }
function Write-Fake($json) { [System.IO.File]::WriteAllText($fake, $json, (New-Object System.Text.UTF8Encoding($false))) }
function Start-Eos {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; $env:EOSOS_FAKE_INDICATORS = $fake
  Start-Process $exe | Out-Null
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Remove-Item env:EOSOS_FAKE_INDICATORS
  Start-Sleep 9
}
$pb = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$P = 0
function Keys { PJs $P "Array.from(document.querySelectorAll('.indicator')).map(function(e){return e.dataset.key}).join(',')" }
function Label { PJs $P "(function(){var e=document.querySelector('.indicator[data-key=battery] .indicator-label');return e?e.textContent:'(nessuna)'})()" }
function Settings-File { if (Test-Path $sf) { Get-Content $sf -Raw | ConvertFrom-Json } else { $null } }
function BarWin { foreach ($b in [IS]::Titled('EosOS')) { $r = New-Object IS+R; [void][IS]::GetWindowRect($b, [ref]$r); if ($r.L -eq $pb.X -and $r.T -eq $pb.Y) { return $b } }; [IntPtr]::Zero }
# Clic destro vero su un punto vuoto della barra (a meta larghezza), solo se sotto il cursore c'e la barra.
function Open-Menu {
  $bar = BarWin; if ($bar -eq [IntPtr]::Zero) { return $false }
  $r = New-Object IS+R; [void][IS]::GetWindowRect($bar, [ref]$r)
  $x = [int](($r.L + $r.Rt) / 2); $y = [int](($r.T + $r.B) / 2)
  [void][IS]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 300
  if ([IS]::Root($x, $y) -ne $bar) { Write-Host '  (clic saltato: sotto il cursore non c e la barra di EosOS)' -ForegroundColor Yellow; return $false }
  [IS]::RightClick(); Start-Sleep -Milliseconds 700
  $true
}
function Menu-Element { [System.Windows.Automation.AutomationElement]::RootElement.FindFirst([System.Windows.Automation.TreeScope]::Children, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ClassNameProperty, '#32768'))) }
function Menu-Items { $m = Menu-Element; if (-not $m) { return @() }; @($m.FindAll([System.Windows.Automation.TreeScope]::Descendants, (New-Object System.Windows.Automation.PropertyCondition([System.Windows.Automation.AutomationElement]::ControlTypeProperty, [System.Windows.Automation.ControlType]::MenuItem)))) }
# Sceglie una voce del menu con un clic vero, dopo aver verificato che sotto il cursore ci sia proprio il menu.
function Choose([string]$name) {
  if (-not (Open-Menu)) { return $false }
  $it = Menu-Items | Where-Object { $_.Current.Name -eq $name } | Select-Object -First 1
  if (-not $it) { Write-Host "  (voce '$name' non trovata)" -ForegroundColor Yellow; return $false }
  $b = $it.Current.BoundingRectangle; $x = [int]($b.X + $b.Width / 2); $y = [int]($b.Y + $b.Height / 2)
  [void][IS]::SetCursorPos($x, $y); Start-Sleep -Milliseconds 300
  $menuHwnd = [IntPtr]((Menu-Element).Current.NativeWindowHandle)
  if ([IS]::Root($x, $y) -ne $menuHwnd) { Write-Host '  (clic saltato: sotto il cursore non c e il menu)' -ForegroundColor Yellow; return $false }
  [IS]::Click(); Start-Sleep -Milliseconds 400
  $true
}
$fake = Join-Path $env:TEMP ('eosos-fake-settings-' + (Get-Random -Maximum 99999) + '.json')
$sf = Join-Path $env:LOCALAPPDATA 'EosOS\indicators.json'
$sfBackup = $null; if (Test-Path $sf) { $sfBackup = Get-Content $sf -Raw }
$sbad = Join-Path $env:LOCALAPPDATA 'EosOS\indicators.json.bad'; $sbadBackup = $null; if (Test-Path $sbad) { $sbadBackup = Get-Content $sbad -Raw; Remove-Item $sbad }
Remove-Item $sf -ErrorAction SilentlyContinue
try {
  Cleanup-Eos; Start-Sleep 1
  Write-Fake '{"battery":{"percent":85,"charging":false,"plugged":false,"minutes_left":135},"network":{"kind":"wifi","online":"internet","name":"Casa","bars":4},"volume":{"percent":40,"muted":false,"device":"Altoparlanti di prova"}}'
  Start-Eos
  foreach ($i in 0..3) { $sx = PJs $i 'window.screenX'; $sy = PJs $i 'window.screenY'; if ($null -ne $sx -and [int]$sx -eq $pb.X -and [int]$sy -eq $pb.Y) { $P = $i } }
  [void][IS]::SetCursorPos($pb.X + 5, $pb.Y + 400)

  Write-Host 'A. predefinito: tutto acceso, percentuale visibile'
  Check 'tre indicatori' (Wait-Until { (Keys) -eq 'battery,network,volume' })
  Check 'percentuale della batteria visibile' ((Label) -eq '85%')
  Check 'nessun file delle impostazioni finche non si sceglie nulla' (-not (Test-Path $sf))
  Check 'il titolo del volume nomina il dispositivo' ((PJs $P "document.querySelector('.indicator[data-key=volume]').title") -like 'Volume 40%*Altoparlanti di prova')

  Write-Host 'B. il menu del clic destro'
  Check 'menu aperto' (Open-Menu)
  $names = (Menu-Items | ForEach-Object { $_.Current.Name }) -join '|'
  Check 'le quattro voci' (($names -like '*Mostra la batteria*') -and ($names -like '*Percentuale della batteria*') -and ($names -like '*Mostra la rete*') -and ($names -like '*audio*')) $names
  # Chiude il menu senza scegliere (Esc) e verifica che non sia cambiato nulla.
  [IS]::Esc(); Start-Sleep -Milliseconds 400
  Check 'chiuso senza scelte: nulla cambia' (((Keys) -eq 'battery,network,volume') -and (-not (Test-Path $sf)))

  Write-Host 'C. percentuale della batteria'
  Check 'scelta fatta' (Choose 'Percentuale della batteria')
  Check 'l etichetta sparisce subito (prima del prossimo controllo)' (Wait-Until { (Label) -eq '(nessuna)' } 1.3)
  Check 'il suggerimento conserva la percentuale' ((PJs $P "document.querySelector('.indicator[data-key=battery]').title") -like 'Batteria 85%*')
  Check 'salvato in indicators.json' ((Settings-File).percent -eq $false -and (Settings-File).battery -eq $true)
  Check 'la scelta si puo rifare: torna la percentuale' ((Choose 'Percentuale della batteria') -and (Wait-Until { (Label) -eq '85%' } 1.3))

  Write-Host 'D. nascondere e rimostrare gli indicatori'
  Check 'nascondo la rete' ((Choose 'Mostra la rete') -and (Wait-Until { (Keys) -eq 'battery,volume' } 1.3))
  Check 'nascondo l audio' ((Choose "Mostra l'audio") -and (Wait-Until { (Keys) -eq 'battery' } 1.3))
  Check 'nascondo la batteria: sparisce tutto il blocco' ((Choose 'Mostra la batteria') -and (Wait-Until { (PJs $P "document.querySelectorAll('.indicators').length") -eq 0 } 1.3))
  Check 'senza indicatori il menu si apre ancora (si puo rimediare)' (Open-Menu)
  [IS]::Esc(); Start-Sleep -Milliseconds 400
  Check 'rimostro la batteria e l audio' ((Choose 'Mostra la batteria') -and (Choose "Mostra l'audio") -and (Wait-Until { (Keys) -eq 'battery,volume' } 1.3))
  $s = Settings-File
  Check 'file: batteria e audio accesi, rete spenta' (($s.battery -eq $true) -and ($s.volume -eq $true) -and ($s.network -eq $false))

  Write-Host 'E. persistenza al riavvio'
  Cleanup-Eos; Start-Sleep 2
  Start-Eos
  Check 'dopo il riavvio la rete resta nascosta' (Wait-Until { (Keys) -eq 'battery,volume' })
  Check 'e la percentuale resta visibile' ((Label) -eq '85%')

  Write-Host 'F. file delle impostazioni illeggibile: si parte da tutto acceso e si conserva una copia'
  Cleanup-Eos; Start-Sleep 2
  [System.IO.File]::WriteAllText($sf, 'questo non e json {', (New-Object System.Text.UTF8Encoding($false)))
  Start-Eos
  Check 'tutto acceso nonostante il file rotto' (Wait-Until { (Keys) -eq 'battery,network,volume' })
  Check 'il file rotto non viene toccato finche non si sceglie' ((Get-Content $sf -Raw) -like 'questo non e json*')
  Check 'una scelta lo sostituisce' (Choose 'Mostra la rete')
  Check 'il file e di nuovo valido' (Wait-Until { (Settings-File).network -eq $false })
  Check 'la copia del file rotto e conservata (.bad)' ((Test-Path $sbad) -and ((Get-Content $sbad -Raw) -like 'questo non e json*'))
}
finally {
  Cleanup-Eos
  Remove-Item $fake -ErrorAction SilentlyContinue
  if ($sfBackup) { [System.IO.File]::WriteAllText($sf, $sfBackup, (New-Object System.Text.UTF8Encoding($false))) } else { Remove-Item $sf -ErrorAction SilentlyContinue }
  if ($sbadBackup) { [System.IO.File]::WriteAllText($sbad, $sbadBackup, (New-Object System.Text.UTF8Encoding($false))) } else { Remove-Item $sbad -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
