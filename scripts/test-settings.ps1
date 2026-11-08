# Verifica S1 delle impostazioni: i comandi settings_all / settings_set / settings_search leggono e scrivono i
# depositi veri (dock.json, indicators.json, launcher.json, mission.json), rifiutano chiavi e valori non ammessi,
# e le modifiche hanno effetto dove ne possono avere. I QUATTRO file dell'utente vengono salvati prima e
# RIPRISTINATI alla fine (compresa l'assenza del file). Nessuna finestra di altri toccata, nessun tasto.
#
# Uso: powershell -File scripts\test-settings.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$expr) { $env:CDP_VIEW = 'launcher'; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
# JSON del valore per le chiamate: true / 'testo' / ['a','b'] come testo JS.
function Set-S([string]$key, [string]$jsValue) { Js "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$key',value:$jsValue})" }
function All-S { @(Js "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) }
function Val([string]$key) { (All-S | Where-Object { $_.key -eq $key }).value }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  # Si parte da file assenti: valori predefiniti, cosi' il confronto e' ripetibile.
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12

  Write-Host 'A. lo stato di tutte le impostazioni'
  $all = All-S
  Check 'ci sono tutte le 18 impostazioni' ($all.Count -eq 18) ("trovate " + $all.Count)
  Check 'ognuna ha un valore' (@($all | Where-Object { $null -eq $_.value }).Count -eq 0) ("senza valore: " + (@($all | Where-Object { $null -eq $_.value } | ForEach-Object { $_.key }) -join ','))
  Check 'predefiniti: Dock non nasconde, menu globale acceso, Alt+Tab e Win+Tab di EosOS, angolo spento' ((Val 'dock.autohide') -eq $false -and (Val 'menu.enabled') -eq $true -and (Val 'mc.alttab') -eq $true -and (Val 'mc.wintab') -eq $true -and (Val 'mc.corner') -eq $false)
  Check 'predefiniti: Launcher Alt+Space, Mission Control Ctrl+Alt+Up' ((Val 'launcher.hotkey') -eq 'Alt+Space' -and (Val 'mc.hotkey') -eq 'Ctrl+Alt+Up')
  $order = ($all | ForEach-Object { $_.section } | Select-Object -Unique) -join ','
  Check 'le sezioni escono nell ordine Aspetto, Dock, Barra dei menu, Launcher, Mission Control' ($order -eq 'appearance,dock,menu_bar,launcher,mission_control') $order

  Write-Host 'B. interruttori: si cambiano, si rileggono, tornano come prima'
  foreach ($k in 'dock.reserve', 'dock.blur', 'bar.battery', 'bar.percent', 'bar.network', 'bar.volume', 'menu.enabled', 'launcher.clipboard', 'mc.corner', 'mc.alttab', 'mc.wintab') {
    $orig = Val $k
    $new = (-not $orig).ToString().ToLower()
    $r = Set-S $k $new
    Check "$k : scritta" ($r.ok -eq $true) ("risposta ok=" + $r.ok + " " + $r.error)
    Check "$k : riletta cambiata" (Wait-Until { (Val $k) -eq (-not $orig) } 4)
    [void](Set-S $k $orig.ToString().ToLower())
    Check "$k : ripristinata" (Wait-Until { (Val $k) -eq $orig } 4)
  }
  $orig = Val 'dock.autohide'; [void](Set-S 'dock.autohide' 'true'); Start-Sleep -Milliseconds 400
  Check 'dock.autohide : acceso e riletto' ((Val 'dock.autohide') -eq $true)
  [void](Set-S 'dock.autohide' 'false'); Start-Sleep -Milliseconds 400
  Check 'dock.autohide : rispento' ((Val 'dock.autohide') -eq $false)

  Write-Host 'C. scelte: livello di vetro e scorciatoia del Launcher'
  foreach ($g in 'clear', 'solid', 'standard') {
    $r = Set-S 'dock.glass' "'$g'"
    Check "vetro $g : accettato e riletto" (($r.ok -eq $true) -and ((Val 'dock.glass') -eq $g))
  }
  $r = Set-S 'dock.glass' "'trasparentissimo'"
  Check 'un livello inventato e rifiutato' (($r.ok -eq $false) -and ($r.error -like '*non ammesso*')) $r.error
  Check 'e lo stato non e cambiato' ((Val 'dock.glass') -eq 'standard')
  $r = Set-S 'launcher.hotkey' "'Ctrl+Alt+Space'"
  Check 'scorciatoia del Launcher Ctrl+Alt+Space : accettata' (($r.ok -eq $true) -and ((Val 'launcher.hotkey') -eq 'Ctrl+Alt+Space')) $r.error
  $r = Set-S 'launcher.hotkey' "'Alt+F4'"
  Check 'Alt+F4 e rifiutata' ($r.ok -eq $false) $r.error
  $r = Set-S 'launcher.hotkey' "'Alt+Space'"
  Check 'torna Alt+Spazio (la predefinita)' (($r.ok -eq $true) -and ((Val 'launcher.hotkey') -eq 'Alt+Space'))
  Check 'e il file non conserva piu la scorciatoia' (-not ((Get-Content (Join-Path $dir 'launcher.json') -Raw) -match 'Ctrl\+Alt\+Space'))

  Write-Host 'D. elenchi: app escluse e cartelle di ricerca'
  $r = Set-S 'menu.excluded' "['Chrome',' discord ','CHROME','']"
  Check 'app escluse : ripulite (spazi, doppi, vuoti)' (($r.ok -eq $true) -and ((@(Val 'menu.excluded')) -join '|') -eq 'Chrome|discord') ((@(Val 'menu.excluded')) -join '|')
  $r = Set-S 'menu.excluded' "'Chrome'"
  Check 'un testo al posto dell elenco e rifiutato' ($r.ok -eq $false)
  [void](Set-S 'menu.excluded' '[]')
  Check 'svuotato' (@(Val 'menu.excluded').Count -eq 0)
  $tmp = $env:TEMP.Replace('\', '\\')
  $r = Set-S 'launcher.folders' "['$tmp']"
  Check 'cartella esistente : accettata' (($r.ok -eq $true) -and (@(Val 'launcher.folders').Count -eq 1)) $r.error
  $r = Set-S 'launcher.folders' "['D:\\cartella-che-non-esiste-eos-xyz']"
  Check 'cartella inesistente : rifiutata con un messaggio' (($r.ok -eq $false) -and ($r.error -like '*non esiste*')) $r.error
  $r = Set-S 'launcher.folders' "['Documenti']"
  Check 'percorso relativo : rifiutato' (($r.ok -eq $false) -and ($r.error -like '*non valida*')) $r.error
  Check 'dopo i rifiuti l elenco e quello di prima' (@(Val 'launcher.folders').Count -eq 1)
  [void](Set-S 'launcher.folders' '[]')
  Check 'svuotato' (@(Val 'launcher.folders').Count -eq 0)

  Write-Host 'E. scorciatoia di Mission Control (si cambia a caldo)'
  $r = Set-S 'mc.hotkey' "'ctrl+alt+k'"
  Check 'scritta in forma canonica e applicata subito' (($r.ok -eq $true) -and ($r.live -eq $true) -and ((Val 'mc.hotkey') -eq 'Ctrl+Alt+K')) ("live=" + $r.live)
  $r = Set-S 'mc.hotkey' "'Su'"
  Check 'un tasto nudo e rifiutato' ($r.ok -eq $false) $r.error
  [void](Set-S 'mc.hotkey' "'Ctrl+Alt+Up'")
  Check 'tornata la predefinita' ((Val 'mc.hotkey') -eq 'Ctrl+Alt+Up')

  Write-Host 'F. sicurezza: chiavi e tipi sbagliati'
  $r = Set-S 'dock.inventata' 'true'
  Check 'chiave sconosciuta rifiutata' (($r.ok -eq $false) -and ($r.error -like '*sconosciuta*'))
  $r = Set-S 'dock.autohide' "'si'"
  Check 'un testo al posto di sì/no rifiutato' ($r.ok -eq $false)
  $r = Set-S '../../mission' 'true'
  Check 'una chiave che sembra un percorso rifiutata' ($r.ok -eq $false)
  $r = Js "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.autohide',value:{a:1}}).then(function(){return 'ok'},function(e){return 'ERR'})"
  Check 'un valore di forma assurda non passa' ($r -eq 'ERR')

  Write-Host 'G. ricerca e avviso di modifica'
  $res = @(Js "window.__TAURI_INTERNALS__.invoke('settings_search',{query:'batteria'})" | ForEach-Object { $_ })
  Check 'cercando batteria la prima e l indicatore' ($res[0] -eq 'bar.battery') ($res -join ',')
  Check 'cercando una parola che non c e non trova nulla' (@(Js "window.__TAURI_INTERNALS__.invoke('settings_search',{query:'zzzz'})" | ForEach-Object { $_ }).Count -eq 0)
  [void](Js "(function(){window.__evs=[];window.__TAURI_INTERNALS__.invoke('plugin:event|listen',{event:'settings-changed',target:{kind:'Any'},handler:window.__TAURI_INTERNALS__.transformCallback(function(e){window.__evs.push(e.payload)})});return 1})()")
  [void](Set-S 'bar.percent' 'false'); Start-Sleep -Milliseconds 500
  $evs = @(Js 'window.__evs' | ForEach-Object { $_ })
  Check 'dopo una modifica parte l evento settings-changed con la chiave' ($evs -contains 'bar.percent') ($evs -join ',')
  [void](Set-S 'bar.percent' 'true')
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
