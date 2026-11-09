# Dock su tutti gli schermi: con EOSOS_DOCK_ALL=1 (nelle build di prova di base c'e' solo il Dock del primario) ogni monitor ha
# il suo Dock, centrato in basso sul proprio monitor, con le stesse app, la sua sfocatura e la sua riserva di spazio; la
# dimensione scelta vale per tutti; spegnendo «Dock su tutti gli schermi» sparisce quello del secondo monitor (e la sua fascia
# riservata) e riaccendendo ricompare. Si legge solo lo stato; non si clicca niente. dock.json e mission.json salvati e
# ripristinati per intero. Serve un PC con due monitor (con uno solo la prova controlla il Dock singolo e lo dice).
#
# Uso: powershell -File scripts\test-dock-multi.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Collections.Generic; using System.Runtime.InteropServices; using System.Text;
public static class DM {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  // Le finestre visibili il cui titolo comincia con «EosOS Dock»: "titolo|L,T,R,B;"
  public static string Docks() {
    string o = "";
    EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); var t = sb.ToString(); if (IsWindowVisible(h) && t.StartsWith("EosOS Dock")) { R r; GetWindowRect(h, out r); o += t + "|" + r.L + "," + r.T + "," + r.Rt + "," + r.B + ";"; } return true; }, IntPtr.Zero);
    return o;
  }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function JsD([int]$i, [string]$expr) { $env:CDP_VIEW = 'dock'; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function JsL([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Dock-Windows { @(([DM]::Docks() -split ';' | Where-Object { $_ }) | ForEach-Object { $p = $_ -split '\|'; $c = $p[1] -split ','; [pscustomobject]@{ title = $p[0]; l = [int]$c[0]; t = [int]$c[1]; r = [int]$c[2]; b = [int]$c[3] } }) }
function Set-S([string]$k, [string]$v) { JsL "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$k',value:$v})" }

$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$screens = [System.Windows.Forms.Screen]::AllScreens
Write-Host "Monitor trovati: $($screens.Count)"
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOSOS_DOCK_ALL = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 14
  $expected = $screens.Count
  Write-Host 'A. un Dock per monitor'
  Check "ci sono $expected finestre Dock" (Wait-Until { @(Dock-Windows).Count -eq $expected } 12) ("trovate: " + @(Dock-Windows).Count)
  $docks = Dock-Windows
  foreach ($s in $screens) {
    $b = $s.Bounds
    $d = $docks | Where-Object { ($_.l + $_.r) / 2 -ge $b.Left -and ($_.l + $_.r) / 2 -lt $b.Right } | Select-Object -First 1
    Check "monitor $($s.DeviceName): c'e' un Dock che sta dentro il suo monitor" ($null -ne $d -and $d.l -ge $b.Left - 1 -and $d.r -le $b.Right + 1 -and $d.b -le $b.Bottom + 1) ($d | Out-String).Trim()
    if ($d) {
      $cx = ($d.l + $d.r) / 2; $mx = ($b.Left + $b.Right) / 2
      Check "monitor $($s.DeviceName): centrato in basso" (([math]::Abs($cx - $mx) -le 30) -and ($d.b -ge $b.Bottom - 80)) "centro Dock $cx, centro monitor $mx, fondo $($d.b)/$($b.Bottom)"
    }
  }
  Write-Host 'B. stesse app e sfocatura su ognuno'
  if ($expected -ge 2) {
    $n0 = [int](JsD 0 "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item:not(.dock-stack)').length")
    $n1 = [int](JsD 1 "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item:not(.dock-stack)').length")
    Check 'i due Dock hanno le stesse app' (($n0 -gt 0) -and ($n0 -eq $n1)) "$n0 e $n1"
    foreach ($i in 0, 1) { Check "Dock ${i}: arriva lo sfondo sfocato" (Wait-Until { (JsD $i "document.documentElement.getAttribute('data-blur')") -eq 'on' } 12) }
    $wa = (JsL "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.all_screens'})[0].value})")
    Check 'l impostazione Dock su tutti gli schermi e accesa di base' ($wa -eq $true)
  }
  Write-Host 'C. la dimensione vale per tutti'
  [void](Set-S 'dock.size' "'large'")
  Start-Sleep -Milliseconds 1500
  foreach ($i in 0..($expected - 1)) { Check "Dock ${i}: icone da 60 px (Grande)" ((JsD $i "getComputedStyle(document.documentElement).getPropertyValue('--dock-icon').trim()") -eq '60px') }
  [void](Set-S 'dock.size' "'medium'")
  Start-Sleep -Milliseconds 800
  Write-Host 'D. spegnere e riaccendere'
  if ($expected -ge 2) {
    [void](Set-S 'dock.all_screens' 'false')
    Check 'resta solo il Dock del primario' (Wait-Until { @(Dock-Windows).Count -eq 1 } 10) ("trovate: " + @(Dock-Windows).Count)
    $left = (Dock-Windows | Select-Object -First 1)
    $prim = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    Check 'ed e quello del monitor primario' (($left.l + $left.r) / 2 -ge $prim.Left -and ($left.l + $left.r) / 2 -lt $prim.Right)
    [void](Set-S 'dock.all_screens' 'true')
    Check 'riaccendendo ricompare quello del secondo monitor' (Wait-Until { @(Dock-Windows).Count -eq $expected } 12) ("trovate: " + @(Dock-Windows).Count)
    Check 'e ha di nuovo le sue app' (Wait-Until { [int](JsD 1 "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item:not(.dock-stack)').length") -gt 0 } 10)
  }
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_DOCK_ALL -ErrorAction SilentlyContinue
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
