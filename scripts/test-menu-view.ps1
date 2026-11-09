# Barra dei menu: le scelte «Mostra il nome dell'app» e «Mostra il menu Finestra» (di base: nome SI, Finestra NO). Si guardano
# solo i pulsanti della barra, qualunque sia l'app in primo piano; non si clicca niente. mission.json salvato e ripristinato.
# Uso: powershell -File scripts\test-menu-view.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Names { (Js 'bar' "Array.from(document.querySelectorAll('.app-menus button')).map(function(b){return b.className.indexOf('app-menu-name')>=0?'NOME':b.textContent.trim()}).join('|')") }
function Set-S([string]$k, [string]$v) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$k',value:$v})" }
function Setting([string]$k) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='$k'})[0].value})" }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Write-Host 'A. di base'
  Check 'il nome dell app e visibile di base' ((Setting 'menu.show_name') -eq $true)
  Check 'il menu Finestra e nascosto di base' ((Setting 'menu.show_window') -eq $false)
  Check 'la barra ha il nome e nessun Finestra' (Wait-Until { $n = Names; ($n -like 'NOME*') -and ($n -notmatch 'Finestra') } 8) (Names)
  Write-Host 'B. nome nascosto'
  $r = Set-S 'menu.show_name' 'false'
  Check 'la scelta si salva e vale subito' (($r.ok -eq $true) -and ($r.live -eq $true))
  Check 'sparisce il nome dell app dalla barra' (Wait-Until { (Names) -notmatch 'NOME' } 6) (Names)
  Write-Host 'C. menu Finestra mostrato'
  [void](Set-S 'menu.show_name' 'true'); [void](Set-S 'menu.show_window' 'true')
  Check 'con Finestra acceso la barra lo mostra (o il nome torna)' (Wait-Until { (Names) -like 'NOME*' } 6) (Names)
  Check 'le due scelte sono nel file' ((Get-Content $mc -Raw) -match 'menuShowWindow')
  Write-Host 'D. valori strani'
  Check 'un valore non booleano e rifiutato' (((Set-S 'menu.show_name' "'forse'").ok) -eq $false)
}
finally {
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
