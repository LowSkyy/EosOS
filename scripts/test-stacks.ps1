# Pile nel Dock: una cartella MIA di prova (file creati qui, in %TEMP%) compare nel Dock come pila; il clic apre il pannello
# SOPRA l'icona con i file piu recenti in griglia; i file di servizio e nascosti non si vedono; un file normale si apre, un
# programma si mostra soltanto (EOSOS_TEST_NO_OPEN registra la richiesta senza lanciare niente); cartella mancante, vuota,
# percorsi di rete rifiutati, al massimo cinque pile. La cartella Download vera non viene mai letta dalla prova (si guarda
# solo che la pila di default ci sia). mission.json viene salvato e ripristinato. Nessun clic su icone vere del Dock.
#
# Uso: powershell -File scripts\test-stacks.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class SK {
  public delegate bool EP(IntPtr h, IntPtr l);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static int[] Rect(long h) { R r; GetWindowRect((IntPtr)h, out r); return new int[] { r.L, r.T, r.Rt, r.B }; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Pn([string]$expr) { Js 'panel' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Panel-Open { [SK]::Find('EosOS Pannello') -ne 0 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }
function Set-Stacks([string]$jsList) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.stacks',value:$jsList})" }
function Stack-Titles { (Js 'dock' "Array.from(document.querySelectorAll('.dock:not(.dock-ghost) .dock-stack')).map(function(e){return e.getAttribute('title')}).join('|')") }
function Open-Stack([int]$i = 0) {
  Close-Panel
  [void](Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-stack')[$i].click()")
  Wait-Until { (Panel-Open) -and ((Pn "document.querySelectorAll('.stack').length") -eq 1) } 6
}
function Stack-Names { (Pn "Array.from(document.querySelectorAll('.stack-file')).map(function(e){return e.getAttribute('title')}).join('|')") }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$root = Join-Path $env:TEMP ("eosos-stacks-" + [guid]::NewGuid().ToString('N').Substring(0, 8))
$dirA = Join-Path $root 'ProvaPila'; $dirEmpty = Join-Path $root 'Vuota'
New-Item -ItemType Directory -Force $dirA, $dirEmpty | Out-Null
try {
  # File miei, con date diverse (il piu recente per primo: nuovo, medio, vecchio).
  $now = Get-Date
  Set-Content (Join-Path $dirA 'vecchio.txt') 'v'; (Get-Item (Join-Path $dirA 'vecchio.txt')).LastWriteTime = $now.AddDays(-9)
  Set-Content (Join-Path $dirA 'medio.txt') 'm'; (Get-Item (Join-Path $dirA 'medio.txt')).LastWriteTime = $now.AddDays(-3)
  Set-Content (Join-Path $dirA 'nuovo.txt') 'n'; (Get-Item (Join-Path $dirA 'nuovo.txt')).LastWriteTime = $now.AddMinutes(-1)
  Set-Content (Join-Path $dirA 'programma.exe') 'MZ'; (Get-Item (Join-Path $dirA 'programma.exe')).LastWriteTime = $now.AddHours(-5)
  New-Item -ItemType Directory -Force (Join-Path $dirA 'SottoCartella') | Out-Null; (Get-Item (Join-Path $dirA 'SottoCartella')).LastWriteTime = $now.AddHours(-2)
  Set-Content (Join-Path $dirA 'desktop.ini') 'x'; Set-Content (Join-Path $dirA '~$bozza.docx') 'x'; Set-Content (Join-Path $dirA 'scaricamento.crdownload') 'x'
  Set-Content (Join-Path $dirA 'nascosto.txt') 'x'; (Get-Item (Join-Path $dirA 'nascosto.txt')).Attributes = 'Hidden'
  Remove-Item $mc -ErrorAction SilentlyContinue
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_NO_OPEN = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. di default c e una sola pila (Download)'
  Check 'una pila nel Dock' (Wait-Until { (Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-stack').length") -eq 1 } 8)
  Check 'si chiama come la cartella Download' ((Stack-Titles) -like 'Download*')
  Check 'ha un separatore prima' ((Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-separator-stacks').length") -eq 1)

  Write-Host 'B. la mia cartella di prova'
  $r = Set-Stacks "['$($dirA.Replace('\','\\'))']"
  Check 'la scelta si salva' ($r.ok -eq $true)
  Check 'il Dock mostra la pila ProvaPila' (Wait-Until { (Stack-Titles) -eq 'ProvaPila' } 6)
  Check 'il pannello si apre' (Open-Stack 0)
  Start-Sleep -Milliseconds 700
  $names = Stack-Names
  Check 'solo i file visibili, dal piu recente: nuovo, SottoCartella, programma, medio, vecchio' ($names -eq 'nuovo.txt|SottoCartella|programma.exe|medio.txt|vecchio.txt') $names
  Check 'file di servizio e nascosti non compaiono' ($names -notmatch 'desktop.ini|bozza|crdownload|nascosto')
  Check 'il titolo e il nome della cartella' ((Pn "document.querySelector('.stack-title').textContent") -eq 'ProvaPila')
  # Posizione: sopra l'icona, dentro il monitor.
  $dpr = [double](Js 'dock' 'window.devicePixelRatio'); $sx = [int](Js 'dock' 'window.screenX'); $sy = [int](Js 'dock' 'window.screenY')
  $ir = Js 'dock' "(function(){var r=document.querySelector('.dock:not(.dock-ghost) .dock-stack').getBoundingClientRect();return [r.left,r.top,r.width,r.height]})()"
  $iconTop = $sy + [int]([math]::Round($ir[1] * $dpr))
  $pr = [SK]::Rect([SK]::Find('EosOS Pannello'))
  $mon = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  Check 'il pannello sta sopra l icona' ($pr[3] -le ($iconTop + 4)) "fondo pannello $($pr[3]), cima icona $iconTop"
  Check 'e dentro lo schermo' (($pr[0] -ge $mon.Left) -and ($pr[2] -le $mon.Right) -and ($pr[1] -ge $mon.Top)) "pannello $($pr -join ',')"
  Check 'ogni pulsante ha un nome' ((Pn "Array.from(document.querySelectorAll('.stack button')).every(function(b){return (b.getAttribute('aria-label')||b.textContent).trim().length>0})") -eq $true)
  Check 'il gruppo ha un nome' ((Pn "document.querySelector('.stack').getAttribute('aria-label')") -eq 'Pila ProvaPila')

  Write-Host 'C. aprire'
  Pn "Array.from(document.querySelectorAll('.stack-file')).filter(function(b){return b.getAttribute('title')==='medio.txt'})[0].click()" | Out-Null
  Check 'un file normale: Rust chiede di aprirlo' (Wait-Until { $o = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('stack_last_open')"; $o -and ($o[0] -like '*medio.txt') -and ($o[1] -eq $false) } 4)
  Check 'il pannello si chiude' (Wait-Until { -not (Panel-Open) } 4)
  [void](Open-Stack 0); Start-Sleep -Milliseconds 600
  Pn "Array.from(document.querySelectorAll('.stack-file')).filter(function(b){return b.getAttribute('title')==='programma.exe'})[0].click()" | Out-Null
  Check 'un programma: solo mostrato in Esplora risorse' (Wait-Until { $o = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('stack_last_open')"; $o -and ($o[0] -like '*programma.exe') -and ($o[1] -eq $true) } 4)
  [void](Open-Stack 0); Start-Sleep -Milliseconds 600
  Pn "document.querySelector('.stack-folder').click()" | Out-Null
  Check 'Apri cartella: apre la cartella della pila' (Wait-Until { $o = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('stack_last_open')"; $o -and ($o[0] -eq $dirA) } 4)
  Check 'un id mai mostrato e rifiutato' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('stack_open',{id:'stk:999'})") -eq $false)
  Check 'un percorso al posto dell id e rifiutato' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('stack_open',{id:'C:\\\\Windows\\\\notepad.exe'})") -eq $false)

  Write-Host 'D. cartella vuota, mancante, troppi file'
  [void](Set-Stacks "['$($dirEmpty.Replace('\','\\'))']"); [void](Wait-Until { (Stack-Titles) -eq 'Vuota' } 6)
  [void](Open-Stack 0); Start-Sleep -Milliseconds 600
  Check 'vuota: lo dice' ((Pn "document.querySelector('.stack-empty').textContent") -like '*vuota*')
  Close-Panel
  $gone = Join-Path $root 'Sparita'
  [void](Set-Stacks "['$($gone.Replace('\','\\'))']"); [void](Wait-Until { (Stack-Titles) -eq 'Sparita' } 6)
  [void](Open-Stack 0); Start-Sleep -Milliseconds 600
  Check 'mancante: avviso (role alert)' ((Pn "document.querySelectorAll('.stack [role=alert]').length") -eq 1)
  Check 'e niente pulsante Apri cartella' ((Pn "document.querySelectorAll('.stack-folder').length") -eq 0)
  Close-Panel
  1..45 | ForEach-Object { Set-Content (Join-Path $dirEmpty ("f{0:D2}.txt" -f $_)) 'x' }
  [void](Set-Stacks "['$($dirEmpty.Replace('\','\\'))']"); [void](Wait-Until { (Stack-Titles) -eq 'Vuota' } 6)
  [void](Open-Stack 0); Start-Sleep -Milliseconds 1500
  Check 'al massimo 40 file mostrati' ((Pn "document.querySelectorAll('.stack-file').length") -eq 40)
  Check 'e dice quanti non si vedono: +5 altri' ((Pn "document.querySelector('.stack-more').textContent") -eq '+5 altri')
  Close-Panel

  Write-Host 'E. impostazione: regole'
  $r = Set-Stacks "['\\\\\\\\server\\\\share']"
  Check 'un percorso di rete e rifiutato' ($r.ok -eq $false) $r.error
  $r = Set-Stacks "['relativo']"
  Check 'un percorso non assoluto e rifiutato' ($r.ok -eq $false)
  $six = (1..6 | ForEach-Object { "'$($root.Replace('\','\\'))\\c$_'" }) -join ','
  [void](Set-Stacks "[$six]")
  Check 'al massimo cinque pile nel Dock' (Wait-Until { (@((Stack-Titles) -split '\|')).Count -eq 5 } 6) (Stack-Titles)
  [void](Set-Stacks '[]')
  Check 'lista vuota: nessuna pila' (Wait-Until { (Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-stack').length") -eq 0 } 6)
  Check 'e niente separatore delle pile' ((Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-separator-stacks').length") -eq 0)
}
finally {
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_NO_OPEN -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
  if ($root -like "$env:TEMP\eosos-stacks-*") { Remove-Item $root -Recurse -Force -ErrorAction SilentlyContinue }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
