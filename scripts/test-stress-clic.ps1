# Stress dei pannelli della barra con clic VERI del mouse a varie velocita': dopo ogni raffica il pannello deve rispondere ancora, cioe' ogni clic singolo deve
# invertire lo stato (aperto <-> chiuso). Clic solo sulla barra di EosOS (build di prova), mai su finestre dell'utente. Usa il mouse vero: non toccarlo mentre gira.
#
# Uso: powershell -File scripts\test-stress-clic.ps1 [-Exe percorso\eosos.exe] [-Delays 30,150,400] [-Clicks 20] [-Targets .clock,.control-btn,.tray-btn]
param([string]$Exe, [string[]]$Delays = @("30", "150", "400"), [int]$Clicks = 20, [string[]]$Targets = @('.clock', '.control-btn'))
$ErrorActionPreference = 'Stop'
# Con -File una lista con la virgola arriva come una sola stringa: si spezza qui.
$DelayList = @($Delays | ForEach-Object { "$_" -split ',' } | ForEach-Object { [int]$_ })
$TargetList = @($Targets | ForEach-Object { "$_" -split ',' })
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -TypeDefinition @'
using System; using System.Runtime.InteropServices;
public static class SC2 { [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c); [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, UIntPtr e);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t); [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
 public static void Click(int x, int y) { SetCursorPos(x, y); mouse_event(2, 0, 0, 0, UIntPtr.Zero); mouse_event(4, 0, 0, 0, UIntPtr.Zero); }
 public static bool PanelVisible() { return IsWindowVisible(FindWindow("Tauri Window", "EosOS Pannello")); } }
'@
[void][SC2]::SetProcessDpiAwarenessContext([IntPtr](-4))
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [int]$idx, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = "$idx"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { & node $cdp eval $expr 2>&1 } finally { $ErrorActionPreference = $old } }
try {
  Cleanup-Eos; Start-Sleep 2
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 14
  foreach ($t in $TargetList) {
    $r = Js 'bar' 1 "(function(){var b=document.querySelector('$t'); if(!b) return 'nessuno'; var r=b.getBoundingClientRect(); return Math.round(r.left+r.width/2)+','+Math.round(r.top+r.height/2)})()"
    if ("$r" -notmatch '(\d+),(\d+)') { Write-Host "  (bersaglio $t assente su questo PC adesso: salto)"; continue }
    $x = [int]$Matches[1]; $y = [int]$Matches[2]
    Write-Host "Bersaglio $t ($x,$y)"
    foreach ($d in $DelayList) {
      for ($i = 0; $i -lt $Clicks; $i++) { [SC2]::Click($x, $y); Start-Sleep -Milliseconds ($d + (Get-Random -Minimum 0 -Maximum ([Math]::Max(1, [int]($d / 4))))) }
      Start-Sleep -Milliseconds 1500
      $bad = 0
      for ($k = 0; $k -lt 4; $k++) {
        $before = [SC2]::PanelVisible(); [SC2]::Click($x, $y); Start-Sleep -Milliseconds 900
        if ($before -eq [SC2]::PanelVisible()) { $bad++; [SC2]::Click($x, $y); Start-Sleep -Milliseconds 900 }
        Start-Sleep -Milliseconds 600
      }
      if ([SC2]::PanelVisible()) { [SC2]::Click($x, $y); Start-Sleep -Milliseconds 700 }
      Check "dopo $Clicks clic ogni ~$d ms ogni clic singolo inverte lo stato" ($bad -eq 0) "(sonde sbagliate: $bad/4)"
    }
  }
}
finally { Cleanup-Eos }
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
