# Barra dei menu con il vetro «liquido»: lo schermo dietro ogni barra (una per monitor) arriva sfocato nel canvas della barra,
# la barra e' esclusa dalle catture dello schermo (come il Dock con la sfocatura) e la tinta si adatta alla luminosita' dello
# sfondo; con la sfocatura spenta (impostazione del Dock) torna l'acrilico di Windows e il canvas si svuota. Si legge solo lo
# stato delle pagine e delle finestre; non si clicca niente. dock.json salvato e ripristinato per intero.
#
# Uso: powershell -File scripts\test-bar-glass.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class BG {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowDisplayAffinity(IntPtr h, out uint a);
  // Tutte le finestre visibili con questo titolo esatto: (handle, affinita').
  public static string Affinities(string title) {
    string o = "";
    EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (IsWindowVisible(h) && sb.ToString() == title) { uint a; GetWindowDisplayAffinity(h, out a); o += a + ","; } return true; }, IntPtr.Zero);
    return o;
  }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function JsB([int]$i, [string]$expr) { $env:CDP_VIEW = 'bar'; $env:CDP_INDEX = "$i"; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function JsL([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Has-Blur([int]$i) { (JsB $i "document.documentElement.getAttribute('data-blur')") -eq 'on' }

$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $blurOn = JsL "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.blur'})[0].value})"
  if ($blurOn -ne $true) { [void](JsL "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:true})"); Start-Sleep 2 }
  $bars = @(0, 1 | Where-Object { $null -ne (JsB $_ 'window.innerWidth') })
  Check 'c e almeno una barra' ($bars.Count -ge 1) "barre: $($bars.Count)"

  Write-Host 'A. sfocatura accesa'
  foreach ($i in $bars) {
    Check "barra ${i}: arriva lo sfondo sfocato (data-blur acceso)" (Wait-Until { Has-Blur $i } 10)
    Check "barra ${i}: il canvas ha dimensioni e disegna qualcosa" ((JsB $i "(function(){var c=document.querySelector('.menubar-blur');if(!c||c.width<10||c.height<3)return false;var d=c.getContext('2d').getImageData(0,0,c.width,c.height).data;var s=0;for(var k=0;k<d.length;k+=64)s+=d[k]+d[k+1]+d[k+2];return s>0})()") -eq $true)
    Check "barra ${i}: e visibile (display non nascosto) e non e tutta trasparente" ((JsB $i "getComputedStyle(document.querySelector('.menubar-blur')).display") -eq 'block')
    Check "barra ${i}: la tinta si adatta (variabile scritta dalla pagina)" (Wait-Until { (JsB $i "document.documentElement.style.getPropertyValue('--bar-blur-bg')") -like 'rgba(*' } 6) (JsB $i "document.documentElement.style.getPropertyValue('--bar-blur-bg')")
    Check "barra ${i}: il testo resta nitido sopra (menubar sopra il canvas)" ((JsB $i "(function(){var m=document.querySelector('.menubar');var c=document.querySelector('.menubar-blur');return parseInt(getComputedStyle(c).zIndex)<0 && getComputedStyle(m).isolation==='isolate'})()") -eq $true)
  }
  $aff = [BG]::Affinities('EosOS')
  Check 'le barre sono escluse dalle catture dello schermo (0x11)' (($aff -split ',' | Where-Object { $_ -ne '' }).Count -ge 1 -and (($aff -split ',' | Where-Object { $_ -ne '' -and $_ -ne '17' }).Count -eq 0)) "affinita': $aff"

  Write-Host 'B. sfocatura spenta'
  [void](JsL "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:false})")
  foreach ($i in $bars) {
    Check "barra ${i}: torna senza vetro catturato (data-blur tolto)" (Wait-Until { -not (Has-Blur $i) } 8)
    Check "barra ${i}: la tinta torna quella di base" ((JsB $i "document.documentElement.style.getPropertyValue('--bar-blur-bg')") -eq '')
  }
  Check 'le barre tornano catturabili (affinita 0)' (Wait-Until { $a = [BG]::Affinities('EosOS'); ($a -split ',' | Where-Object { $_ -ne '' -and $_ -ne '0' }).Count -eq 0 } 8) ([BG]::Affinities('EosOS'))

  Write-Host 'C. di nuovo accesa'
  [void](JsL "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:true})")
  foreach ($i in $bars) { Check "barra ${i}: il vetro torna" (Wait-Until { Has-Blur $i } 10) }
}
finally {
  Cleanup-Eos
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
