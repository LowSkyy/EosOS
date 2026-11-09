# Misura quanto ci mette a comparire il pannello sotto l'orologio (e il contenuto) dopo che e' rimasto nascosto, con e senza la
# riduzione di memoria delle pagine nascoste (EOSOS_WEBMEM=0 la spegne). Si clicca solo l'orologio della barra di EosOS.
# Uso: powershell -File scripts\measure-open.ps1 [-Exe percorso\eosos.exe] [-Rounds 5]
param([string]$Exe, [int]$Rounds = 5)
$ErrorActionPreference = 'Stop'
if (-not $Exe) { $Exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class MO { public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l); [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n); [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 public static bool PanelOpen(){ bool f=false; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString().Contains("EosOS Pannello")){f=true;return false;} return true;},IntPtr.Zero); return f; } }
'@
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bk = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
function Run([string]$label, [string]$webmem) {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:EOSOS_WEBMEM = $webmem
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $Exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 14
  $times = @()
  for ($i = 0; $i -lt $Rounds; $i++) {
    [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
    Start-Sleep 8   # nascosto abbastanza a lungo perche' la riduzione di memoria faccia effetto
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    [void](Js 'bar' "document.querySelector('.clock').click()")
    while (-not ([MO]::PanelOpen() -and ((Js 'panel' "document.querySelectorAll('.cal-day').length") -eq 42)) -and $sw.ElapsedMilliseconds -lt 6000) { Start-Sleep -Milliseconds 20 }
    $times += $sw.ElapsedMilliseconds
  }
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
  Remove-Item env:EOSOS_WEBMEM -ErrorAction SilentlyContinue
  Write-Host ("{0,-34} aperture (ms, include il polling di prova): {1}   mediana {2}" -f $label, ($times -join ' '), (($times | Sort-Object)[[int]($times.Count / 2)]))
}
try {
  Run 'con riduzione di memoria (normale)' '1'
  Run 'senza riduzione (EOSOS_WEBMEM=0)' '0'
}
finally {
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bk) { [System.IO.File]::WriteAllBytes($mc, $bk) }
}

# La build di produzione nasconde la barra di Windows e fermarla a forza (tutti gli eosos.exe, watchdog compreso) non la ripristina: si rimette qui.
& (Join-Path $PSScriptRoot "restore-taskbar.ps1")
