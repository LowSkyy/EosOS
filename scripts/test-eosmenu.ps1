# Menu EosOS (il «menu mela» a sinistra della barra): il pulsante apre il pannello con le voci; Informazioni mostra la versione; Impostazioni di
# EosOS apre la finestra delle impostazioni; Spegni e Riavvia nelle build di prova NON eseguono niente (scrivono solo una riga in
# %TEMP%\eosos-actions.log, che viene salvato e ripristinato); Esci da EosOS non si clicca. Si guarda solo lo stato delle pagine.
#
# Uso: powershell -File scripts\test-eosmenu.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class EM { public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l); [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n); [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 public static bool Visible(string part){ bool f=false; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString().Contains(part)){f=true;return false;} return true;},IntPtr.Zero); return f; } }
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Pn([string]$e) { Js 'panel' $e }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Open-Menu { Js 'bar' "document.querySelector('.eos-btn').click()" | Out-Null; Wait-Until { ([EM]::Visible('EosOS Pannello')) -and ((Pn "document.querySelectorAll('.eosmenu-item').length") -ge 10) } 6 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not ([EM]::Visible('EosOS Pannello')) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }
function Click-Item([string]$id) { Pn "document.querySelector('.eosmenu-item[data-index=\'$id\']').click()" | Out-Null }
$log = Join-Path $env:TEMP 'eosos-actions.log'
$bkLog = if (Test-Path $log) { [System.IO.File]::ReadAllBytes($log) } else { $null }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc, $log -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. il pulsante e il pannello'
  Check 'c e il pulsante a sinistra della barra, prima dei menu dell app' ((Js 'bar' "(function(){var l=document.querySelector('.menubar-left');return l.children[0].classList.contains('eos-btn')})()") -eq $true)
  Check 'ha un nome per i lettori di schermo' ((Js 'bar' "document.querySelector('.eos-btn').getAttribute('aria-label')") -eq 'Menu EosOS')
  Check 'un clic apre il pannello' (Wait-Until { (Js 'bar' "document.querySelector('.eos-btn').click()") | Out-Null; ([EM]::Visible('EosOS Pannello')) -and ((Pn "document.querySelectorAll('.eosmenu-item').length") -ge 1) } 8)
  Check 'ha UNA sola voce' ((Pn "document.querySelectorAll('.eosmenu-item').length") -eq 1)
  $label = Pn "document.querySelector('.eosmenu-item span:first-child').textContent"
  Check 'la voce e «Impostazioni»' ($label -like 'Impostazioni*') $label
  Check 'il pannello e un menu per i lettori di schermo' ((Pn "document.querySelector('.eosmenu').getAttribute('role')") -eq 'menu')
  Check 'la voce ha il focus' (Wait-Until { (Pn "document.activeElement && document.activeElement.getAttribute('data-index')") -eq '0' } 3)

  Write-Host 'B. la voce apre le impostazioni'
  Click-Item 0
  Check 'si apre la finestra delle impostazioni' (Wait-Until { [EM]::Visible('EosOS Impostazioni') } 6)
  Check 'e il pannello si chiude' (Wait-Until { -not ([EM]::Visible('EosOS Pannello')) } 4)
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_close')"); Start-Sleep 2

  Write-Host 'C. altre finestre (controllo acceso)'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Check 'dalla barra eosmenu_run e rifiutato' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('eosmenu_run',{id:'shutdown'}).then(function(r){return r},function(e){return 'ERR'})") -eq $false)
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-Item $log, $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkLog) { [System.IO.File]::WriteAllBytes($log, $bkLog) }
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
