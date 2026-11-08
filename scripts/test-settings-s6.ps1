# Verifica S6 delle impostazioni: tastiera e accessibilita della finestra. Frecce su/giu nella barra laterale (si ferma
# ai bordi, il focus segue), il focus iniziale e sulla ricerca, ogni controllo ha un nome accessibile e l'aiuto e collegato
# (aria-describedby), un valore personalizzato nel file non lascia una tendina vuota. I quattro file delle impostazioni
# vengono salvati e ripristinati. Nessuna finestra di altri toccata, nessun tasto vero (eventi di tastiera nella pagina).
#
# Uso: powershell -File scripts\test-settings-s6.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Win([string]$expr) { Js 'settings' $expr }
function Heading { Win "document.querySelector('.settings-h').textContent" }
function Active { Win "(document.activeElement.dataset||{}).section || document.activeElement.className" }
function Key([string]$k) { Win "document.activeElement.dispatchEvent(new KeyboardEvent('keydown',{key:'$k',bubbles:true,cancelable:true}))" | Out-Null; Start-Sleep -Milliseconds 400 }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  # Una scorciatoia del Launcher scritta a mano, fuori dalle tre opzioni.
  [System.IO.File]::WriteAllText((Join-Path $dir 'launcher.json'), '{"version":1,"usage":{},"hotkey":"Ctrl+Alt+Z"}')
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:null})"); Start-Sleep -Milliseconds 1200

  Write-Host 'A. focus e tastiera'
  Check 'all apertura il focus e sulla ricerca' ((Win "document.activeElement.className") -eq 'settings-search')
  Win "document.querySelector('[data-section=general]').focus()" | Out-Null
  Key 'ArrowDown'
  Check 'freccia giu: passa ad Aspetto e il focus la segue' (((Heading) -eq 'Aspetto') -and ((Active) -eq 'appearance')) ((Heading) + ' / ' + (Active))
  Key 'ArrowUp'; Key 'ArrowUp'
  Check 'freccia su: torna a Generale e li si ferma (non gira)' (((Heading) -eq 'Generale') -and ((Active) -eq 'general')) ((Heading) + ' / ' + (Active))
  $n = [int](Win "document.querySelectorAll('[data-section]').length")
  Win "document.querySelector('[data-section=mission_control]').focus()" | Out-Null
  Key 'ArrowDown'
  Check 'in fondo la freccia giu non fa niente' (((Heading) -eq 'Mission Control') -and ($n -eq 6)) ((Heading) + " sezioni $n")

  Write-Host 'B. accessibilita di ogni sezione'
  $bad = @()
  foreach ($sec in 'general', 'appearance', 'dock', 'menu_bar', 'launcher', 'mission_control') {
    Win "document.querySelector('[data-section=$sec]').click()" | Out-Null; Start-Sleep -Milliseconds 400
    $r = Win "(function(){var out=[];document.querySelectorAll('.settings-main button, .settings-main select, .settings-main input').forEach(function(el){var name=el.getAttribute('aria-label')||(el.textContent||'').trim()||el.getAttribute('title')||'';if(!name&&el.id){var l=document.querySelector('label[for='+el.id+']');if(l)name=l.textContent.trim()}if(!name)out.push(el.tagName+'.'+el.className)});return out.join(',')})()"
    if ($r) { $bad += "$sec : $r" }
  }
  Check 'ogni pulsante, scelta e campo ha un nome accessibile' ($bad.Count -eq 0) ($bad -join ' ; ')
  Win "document.querySelector('[data-section=dock]').click()" | Out-Null; Start-Sleep -Milliseconds 300
  Check 'gli interruttori puntano al loro aiuto con aria-describedby' ((Win "Array.from(document.querySelectorAll('[role=switch]')).every(function(s){var d=s.getAttribute('aria-describedby');return d && document.getElementById(d) && document.getElementById(d).textContent.length>3})") -eq $true)
  Check 'gli interruttori hanno aria-checked vero o falso' ((Win "Array.from(document.querySelectorAll('[role=switch]')).every(function(s){return s.getAttribute('aria-checked')==='true'||s.getAttribute('aria-checked')==='false'})") -eq $true)
  Check 'la finestra e un dialogo con un nome e le sezioni sono una navigazione' (((Win "document.querySelector('[role=dialog]').getAttribute('aria-label')") -eq 'Impostazioni di EosOS') -and ((Win "document.querySelector('nav').getAttribute('aria-label')") -eq 'Sezioni'))
  Check 'la sezione attuale e segnata con aria-current' ((Win "document.querySelectorAll('[aria-current=page]').length") -eq 1)

  Write-Host 'C. un valore personalizzato nel file non lascia la tendina vuota'
  Win "document.querySelector('[data-section=launcher]').click()" | Out-Null; Start-Sleep -Milliseconds 500
  $sel = Win "(function(){var s=document.querySelector('select');return s.options.length+'|'+s.options[s.selectedIndex].textContent})()"
  Check 'la scorciatoia scritta a mano compare come quarta opzione, selezionata' ($sel -like '4|Ctrl+Alt+Z*personalizzata*') $sel
}
finally {
  Cleanup-Eos
  foreach ($f in $files) {
    $p = Join-Path $dir $f
    Remove-Item $p -ErrorAction SilentlyContinue
    if ($null -ne $backup[$f]) { [System.IO.File]::WriteAllBytes($p, $backup[$f]) }
  }
}
$same = $true
foreach ($f in $files) { $p = Join-Path $dir $f; $now = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null }; if (($null -eq $now) -ne ($null -eq $backup[$f])) { $same = $false } elseif ($null -ne $now -and -not [System.Linq.Enumerable]::SequenceEqual([byte[]]$now, [byte[]]$backup[$f])) { $same = $false } }
Write-Host ("File dell utente ripristinati identici: " + $same)
if ($script:fail -eq 0 -and $same) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
