# Togliere un'app dal Dock: l'icona deve sparire SUBITO (prima si aggiornava solo al giro successivo, fino a 2 secondi dopo, e
# il Dock sembrava bloccato). Si toglie l'ultima app fissata passando dalle Impostazioni (lo stesso percorso di "Rimuovi dal
# Dock": stessa funzione di Rust e stesso evento) e si misura dopo quanto sparisce dalla pagina. dock.json dell'utente viene
# SALVATO per intero e RIPRISTINATO (anche se il test fallisce), e il Dock lo rilegge senza riavvio.
#
# Uso: powershell -File scripts\test-dock-remove.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Count-Items { [int](Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item:not(.dock-stack)').length") }

$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
  $pinned = @(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.pinned'})[0].value})" | ForEach-Object { $_ })
  Check 'ci sono almeno due app fissate per provare' ($pinned.Count -ge 2) "fissate: $($pinned.Count)"
  if ($pinned.Count -ge 2) {
    $before = Count-Items
    $keep = ($pinned | Select-Object -First ($pinned.Count - 1) | ForEach-Object { "'" + ($_ -replace "'", "\'") + "'" }) -join ','
    Start-Sleep -Milliseconds 2500   # lascia passare il giro periodico, cosi la misura parte da uno stato fermo
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.pinned',value:[$keep]})")
    $setMs = $sw.ElapsedMilliseconds
    $gone = $false
    while ($sw.ElapsedMilliseconds -lt 4000 -and -not $gone) { if ((Count-Items) -lt $before) { $gone = $true } else { Start-Sleep -Milliseconds 40 } }
    $ms = $sw.ElapsedMilliseconds
    Write-Host "  (settings_set ha risposto in $setMs ms; l'icona e sparita dopo $ms ms)"
    Check 'l icona sparisce dal Dock' $gone
    Check 'e sparisce subito (meno di un secondo, non al giro periodico di 2 s)' ($gone -and $ms -lt 1000) "$ms ms"
    Check 'l app non e piu fra le fissate' ((@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.pinned'})[0].value})" | ForEach-Object { $_ })).Count -eq ($pinned.Count - 1))
  }
}
finally {
  Cleanup-Eos
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) } else { Remove-Item $mc -ErrorAction SilentlyContinue }
  Write-Host ("dock.json ripristinato: " + ((Test-Path $dj) -and ([System.IO.File]::ReadAllBytes($dj).Length -eq $bkDj.Length)))
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
