# Launchpad (tutte le app in griglia, a tutto schermo): l'icona in testa al Dock e la voce del menu EosOS lo aprono; la finestra copre
# l'area di lavoro del monitor; le app sono in ordine alfabetico e arrivano a pagine; la ricerca filtra (senza badare ad accenti
# e maiuscole); Esc chiude; un clic sull'icona di Blocco note ne avvia UNO NUOVO (si chiude solo quello nato durante la prova).
# Non si clicca su icone vere del Dock: solo sul Launchpad e sulle sue tessere. mission.json e launcher.json salvati e ripristinati.
#
# Uso: powershell -File scripts\test-launchpad.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class LP { public delegate bool EP(IntPtr h, IntPtr l);
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l); [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n); [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h); [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 public static int[] Rect(string title){ int[] o=null; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString()==title){R r;GetWindowRect(h,out r);o=new int[]{r.L,r.T,r.Rt,r.B};return false;} return true;},IntPtr.Zero); return o; } }
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Pad-Open { $null -ne [LP]::Rect('EosOS Launcher') }
function Apps { [int](Js 'launcher' "document.querySelectorAll('.launchpad-app').length") }
function Type-Search([string]$text) { Js 'launcher' "(function(){var set=Object.getOwnPropertyDescriptor(HTMLInputElement.prototype,'value').set;var i=document.querySelector('.launchpad-search');set.call(i,'$text');i.dispatchEvent(new Event('input',{bubbles:true}));return 1})()" | Out-Null; Start-Sleep -Milliseconds 400 }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$lj = Join-Path $env:LOCALAPPDATA 'EosOS\launcher.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$bkLj = if (Test-Path $lj) { [System.IO.File]::ReadAllBytes($lj) } else { $null }
$notepadBefore = @(Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 14

  Write-Host 'A. aprirlo dal Dock'
  Check 'l icona del Launchpad e la prima del Dock' ((Js 'dock' "(function(){var f=document.querySelector('.dock:not(.dock-ghost) .dock-item');return f.classList.contains('dock-launchpad')&&f.getAttribute('aria-label')==='Launchpad'})()") -eq $true)
  Js 'dock' "document.querySelector('.dock-launchpad').click()" | Out-Null
  Check 'si apre la finestra del Launchpad' (Wait-Until { Pad-Open } 6)
  Start-Sleep -Milliseconds 800
  $r = [LP]::Rect('EosOS Launcher')
  # Si apre sul monitor dove sta il cursore: basta che copra l'area di lavoro di UNO dei monitor.
  $covers = $false
  foreach ($scr in [System.Windows.Forms.Screen]::AllScreens) { $w = $scr.WorkingArea; if ($r -and ([math]::Abs(($r[2] - $r[0]) - $w.Width) -le 4) -and ([math]::Abs(($r[3] - $r[1]) - $w.Height) -le 4) -and ([math]::Abs($r[0] - $w.Left) -le 4)) { $covers = $true } }
  Check 'copre l area di lavoro di un monitor (quello del cursore)' $covers ("finestra " + ($r -join ','))
  Check 'la griglia e visibile (data-grid)' ((Js 'launcher' "document.querySelector('.launcher').getAttribute('data-grid')") -eq 'true')
  Check 'le app arrivano (almeno 10)' (Wait-Until { (Apps) -ge 10 } 12) ("app: " + (Apps))
  $names = @(Js 'launcher' "Array.from(document.querySelectorAll('.launchpad-name')).map(function(e){return e.textContent})" | ForEach-Object { $_ })
  $sorted = @($names | Sort-Object { $_.ToLowerInvariant().Normalize([Text.NormalizationForm]::FormD) })
  Check 'sono in ordine alfabetico' (($names -join '|').ToLowerInvariant() -eq ($sorted -join '|').ToLowerInvariant()) ($names | Select-Object -First 5) -join ','
  $bad = @($names | Where-Object { $_ -match '^(Prompt dei comandi|Windows PowerShell|Python 3|Node\.js|Esegui|Cestino|Editor del Registro|Visualizzatore eventi|Global Flags|Application Verifier)' })
  Check 'niente prompt dei comandi, PowerShell, Python, strumenti di sistema o debugger' ($bad.Count -eq 0) ($bad -join ', ')
  $docs = @($names | Where-Object { $_ -match '(License|Licenza|Manual|Documentation|Website|Uninstall|Disinstalla| Help$)' })
  Check 'niente manuali, licenze, siti o disinstallatori' ($docs.Count -eq 0) ($docs -join ', ')
  Check 'ci sono i puntini delle pagine (se le pagine sono piu di una)' ((Js 'launcher' "(function(){var p=document.querySelectorAll('.launchpad-page').length;var d=document.querySelectorAll('.launchpad-dot').length;return p===d})()") -eq $true)
  $pages = [int](Js 'launcher' "document.querySelectorAll('.launchpad-page').length")
  if ($pages -ge 2) {
    Js 'launcher' "document.querySelector('.launchpad').dispatchEvent(new KeyboardEvent('keydown',{key:'PageDown',bubbles:true}))" | Out-Null
    Start-Sleep -Milliseconds 600
    Check 'PagGiu passa alla seconda pagina' ((Js 'launcher' "document.querySelector('.launchpad-dot[data-active='true']')===document.querySelectorAll('.launchpad-dot')[1]") -eq $true)
    Js 'launcher' "document.querySelector('.launchpad').dispatchEvent(new KeyboardEvent('keydown',{key:'PageUp',bubbles:true}))" | Out-Null
    Start-Sleep -Milliseconds 600
    Check 'PagSu torna alla prima' ((Js 'launcher' "document.querySelector('.launchpad-dot[data-active='true']')===document.querySelectorAll('.launchpad-dot')[0]") -eq $true)
  }
  Check 'il campo di ricerca ha un nome' ((Js 'launcher' "document.querySelector('.launchpad-search').getAttribute('aria-label')") -eq 'Cerca tra le app')
  Check 'la griglia e un elenco per i lettori di schermo' ((Js 'launcher' "document.querySelector('.launchpad-grid').getAttribute('role')") -eq 'listbox')

  Write-Host 'B. ricerca'
  # Si aspetta che tutte le pagine di app siano arrivate (il numero smette di crescere).
  $prev = -1; while ((Apps) -ne $prev) { $prev = Apps; Start-Sleep -Milliseconds 1200 }
  $all = Apps
  Type-Search 'zzzzqq'
  Check 'una ricerca senza risultati lo dice' (((Apps) -eq 0) -and ((Js 'launcher' "(document.querySelector('.launchpad-empty')||{textContent:''}).textContent") -like 'Nessuna app*'))
  Type-Search 'blocco'
  Check 'cercando blocco resta (almeno) Blocco note' (((Apps) -ge 1) -and ((Js 'launcher' "Array.from(document.querySelectorAll('.launchpad-name')).some(function(e){return e.textContent.toLowerCase().indexOf('blocco note')>=0})") -eq $true)) ("app: " + (Apps))
  Type-Search ''
  Check 'svuotando il campo tornano tutte' ((Apps) -eq $all)

  Write-Host 'C. avviare un app (Blocco note, uno nuovo mio)'
  Type-Search 'blocco note'
  Js 'launcher' "Array.from(document.querySelectorAll('.launchpad-app')).filter(function(b){return b.getAttribute('aria-label').toLowerCase()==='blocco note'})[0].click()" | Out-Null
  Check 'parte un nuovo Blocco note' (Wait-Until { @(Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $notepadBefore -notcontains $_.Id }).Count -ge 1 } 15)
  Check 'e il Launchpad si chiude' (Wait-Until { -not (Pad-Open) } 5)

  Write-Host 'D. dal menu EosOS'
  Js 'bar' "document.querySelector('.eos-btn').click()" | Out-Null
  Start-Sleep 1
  Js 'panel' "Array.from(document.querySelectorAll('.eosmenu-item')).filter(function(b){return b.textContent.indexOf('Launchpad')>=0})[0].click()" | Out-Null
  Check 'la voce Launchpad del menu lo apre' (Wait-Until { (Pad-Open) -and ((Apps) -ge 10) } 10)

  Write-Host 'E. chiusura e ritorno alla ricerca normale'
  Js 'launcher' "window.__TAURI_INTERNALS__.invoke('launcher_hide')" | Out-Null
  Check 'si chiude' (Wait-Until { -not (Pad-Open) } 5)
  Check 'la modalita Launchpad non resta (data-grid tolto)' (Wait-Until { (Js 'launcher' "document.querySelector('.launcher').getAttribute('data-grid')") -ne 'true' } 4)

  Write-Host 'F. altre finestre (controllo acceso)'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Check 'dalla barra launcher_grid non da niente' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('launcher_grid',{offset:0,limit:5,have:[]}).then(function(r){return r.total},function(e){return -1})") -eq 0)
  Js 'bar' "window.__TAURI_INTERNALS__.invoke('dock_launchpad')" | Out-Null
  Start-Sleep -Milliseconds 1500
  Check 'dalla barra dock_launchpad non apre niente' (-not (Pad-Open))
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $notepadBefore -notcontains $_.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
  Remove-Item $mc, $lj -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
  if ($null -ne $bkLj) { [System.IO.File]::WriteAllBytes($lj, $bkLj) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
