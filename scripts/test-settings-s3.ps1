# Verifica S3 delle impostazioni: scorciatoia di Mission Control cambiata a caldo (registrazione dalla pagina, rifiuto
# dei conflitti), cartelle di ricerca col selettore di Windows, app fissate (riordino e rimozione), sezione Aspetto.
# I QUATTRO file delle impostazioni dell'utente vengono salvati e RIPRISTINATI alla fine; per le app fissate si usa
# un dock.json di prova con tre programmi di Windows. I tasti veri sono solo Ctrl+Alt+<tasto> per provare la
# scorciatoia (mentre la finestra delle impostazioni di EosOS e in primo piano); nessuna finestra di altri toccata.
# Il selettore di Windows si apre davvero ma si chiude subito con WM_CLOSE (nessuna cartella viene scelta li').
#
# Uso: powershell -File scripts\test-settings-s3.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class S3 {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
  public static void Combo(byte vk) { keybd_event(0x11, 0, 0, UIntPtr.Zero); keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); keybd_event(0x11, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Win([string]$expr) { Js 'settings' $expr }
function Inv([string]$expr) { Js 'launcher' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Val([string]$key) { (@(Inv "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq $key }).value }
function Click-Nav([string]$name) { Win "(Array.from(document.querySelectorAll('.settings-nav')).find(function(e){return e.textContent==='$name'})||{click:function(){}}).click()" | Out-Null; Start-Sleep -Milliseconds 400 }
function Press([string]$init) { Win "document.body.dispatchEvent(new KeyboardEvent('keydown',{$init,bubbles:true,cancelable:true}))" | Out-Null }
function Mc-Phase { (Inv "window.__TAURI_INTERNALS__.invoke('mc_state')").phase }
function Close-Mc { [void](Inv "window.__TAURI_INTERNALS__.invoke('mc_close')"); Wait-Until { (Mc-Phase) -eq 'closed' } 4 | Out-Null; Start-Sleep -Milliseconds 400 }
function Focus-Settings { [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'mission_control'})"); Start-Sleep -Milliseconds 800 }
function Start-Eos { $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12 }

$dir = Join-Path $env:LOCALAPPDATA 'EosOS'
$files = 'dock.json', 'indicators.json', 'launcher.json', 'mission.json'
$backup = @{}
foreach ($f in $files) { $p = Join-Path $dir $f; $backup[$f] = if (Test-Path $p) { [System.IO.File]::ReadAllBytes($p) } else { $null } }
try {
  Cleanup-Eos; Start-Sleep 1
  foreach ($f in $files) { Remove-Item (Join-Path $dir $f) -ErrorAction SilentlyContinue }
  # Un Dock di prova con tre programmi di Windows fissati.
  $pins = '{"version":1,"pinned":[{"name":"Alfa","path":"C:\\Windows\\System32\\notepad.exe"},{"name":"Beta","path":"C:\\Windows\\System32\\cmd.exe"},{"name":"Gamma","path":"C:\\Windows\\System32\\calc.exe"}]}'
  [System.IO.File]::WriteAllText((Join-Path $dir 'dock.json'), $pins)
  $tmp = $env:TEMP
  $env:EOSOS_TEST_PICK_FOLDER = $tmp   # solo per la prima parte: il selettore finto restituisce questa cartella
  Start-Eos
  [void](Inv "window.__TAURI_INTERNALS__.invoke('settings_open',{section:null})"); Start-Sleep -Milliseconds 1200
  Check 'la finestra delle impostazioni e aperta' ([S3]::Find('EosOS Impostazioni') -ne 0)

  Write-Host 'A. sezione Aspetto'
  Click-Nav 'Aspetto'
  Check 'ha sfocatura e livello di vetro' ((Win "document.querySelector('.settings-h').textContent") -eq 'Aspetto' -and (Win "document.querySelectorAll('.settings-row').length") -eq 2)

  Write-Host 'B. app fissate: riordino e rimozione'
  Click-Nav 'Dock'
  Check 'le tre app fissate nell ordine del file' (((Win "Array.from(document.querySelectorAll('.pinned-name')).map(function(e){return e.textContent})") -join '|') -eq 'Alfa|Beta|Gamma') ((Win "Array.from(document.querySelectorAll('.pinned-name')).map(function(e){return e.textContent})") -join '|')
  Check 'la prima non ha la freccia su e l ultima non ha la freccia giu' ((Win "document.querySelectorAll('.pinned-item')[0].querySelectorAll('button')[0].disabled") -eq $true -and (Win "document.querySelectorAll('.pinned-item')[2].querySelectorAll('button')[1].disabled") -eq $true)
  Win "document.querySelectorAll('.pinned-item')[0].querySelectorAll('button')[1].click()" | Out-Null
  Check 'giu su Alfa: l ordine diventa Beta, Alfa, Gamma nel deposito' (Wait-Until { ((@(Val 'dock.pinned')) -join '|') -eq 'Beta|Alfa|Gamma' } 4) ((@(Val 'dock.pinned')) -join '|')
  Check 'e anche nel file del Dock' (Wait-Until { $j = Get-Content (Join-Path $dir 'dock.json') -Raw; ($j.IndexOf('Beta') -lt $j.IndexOf('Alfa')) -and ($j.IndexOf('Alfa') -lt $j.IndexOf('Gamma')) } 4)
  Win "document.querySelectorAll('.pinned-item')[1].querySelectorAll('button')[0].click()" | Out-Null
  Check 'su su Alfa: torna Alfa, Beta, Gamma' (Wait-Until { ((@(Val 'dock.pinned')) -join '|') -eq 'Alfa|Beta|Gamma' } 4)
  Win "document.querySelectorAll('.pinned-item')[1].querySelectorAll('button')[2].click()" | Out-Null
  Check 'togliere Beta la toglie dal Dock (resta Alfa, Gamma)' (Wait-Until { ((@(Val 'dock.pinned')) -join '|') -eq 'Alfa|Gamma' } 4) ((@(Val 'dock.pinned')) -join '|')
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.pinned',value:['Alfa','Inventata']})"
  Check 'un nome che non e nel Dock viene rifiutato e non cambia nulla' (($r.ok -eq $false) -and ((@(Val 'dock.pinned')) -join '|') -eq 'Alfa|Gamma') $r.error

  Write-Host 'C. cartelle di ricerca (selettore finto per la parte di pagina)'
  Click-Nav 'Launcher'
  Check 'all inizio nessuna cartella' ((Win "document.querySelectorAll('.names-item').length") -eq 0)
  Win "Array.from(document.querySelectorAll('.settings-button')).find(function(b){return b.textContent.indexOf('Aggiungi cartella')===0}).click()" | Out-Null
  Check 'Aggiungi cartella aggiunge quella scelta' (Wait-Until { @(Val 'launcher.folders').Count -eq 1 } 5) ((@(Val 'launcher.folders')) -join ',')
  Check 'compare in pagina con l ultimo pezzo del percorso' (Wait-Until { (Win "document.querySelectorAll('.names-item')[0].textContent") -like ((Split-Path $tmp -Leaf) + '*') } 4)
  Check 'e nel file del Launcher' ((Get-Content (Join-Path $dir 'launcher.json') -Raw) -match 'searchFolders|search_folders')
  Win "Array.from(document.querySelectorAll('.settings-button')).find(function(b){return b.textContent.indexOf('Aggiungi cartella')===0}).click()" | Out-Null
  Start-Sleep -Milliseconds 800
  Check 'scegliere due volte la stessa cartella non la duplica' (@(Val 'launcher.folders').Count -eq 1)
  Win "document.querySelector('.names-item button').click()" | Out-Null
  Check 'la x la toglie' (Wait-Until { @(Val 'launcher.folders').Count -eq 0 } 4)

  Write-Host 'D. scorciatoia di Mission Control: registrazione dalla pagina'
  Click-Nav 'Mission Control'
  Check 'mostra la scorciatoia attuale' ((Win "document.querySelector('.hotkey-keys').textContent") -eq 'Ctrl+Alt+Up')
  Win "Array.from(document.querySelectorAll('.hotkey .settings-button')).find(function(b){return b.textContent==='Cambia'}).click()" | Out-Null; Start-Sleep -Milliseconds 300
  Check 'Cambia mette la pagina in ascolto' ((Win "document.querySelector('.hotkey-keys').textContent") -like 'Premi la nuova*')
  Press "key:'k',code:'KeyK'"; Start-Sleep -Milliseconds 300
  Check 'un tasto da solo non basta: resta in ascolto' ((Win "document.querySelector('.hotkey-keys').textContent") -like 'Premi la nuova*')
  Press "key:'Escape',code:'Escape'"; Start-Sleep -Milliseconds 300
  Check 'Esc annulla la registrazione senza chiudere la finestra' (((Win "document.querySelector('.hotkey-keys').textContent") -eq 'Ctrl+Alt+Up') -and ([S3]::Find('EosOS Impostazioni') -ne 0))
  Win "Array.from(document.querySelectorAll('.hotkey .settings-button')).find(function(b){return b.textContent==='Cambia'}).click()" | Out-Null; Start-Sleep -Milliseconds 300
  Press "key:'k',code:'KeyK',ctrlKey:true,altKey:true"
  Check 'Ctrl+Alt+K viene accettata' (Wait-Until { (Val 'mc.hotkey') -eq 'Ctrl+Alt+K' } 4)
  Check 'e la pagina la mostra' ((Win "document.querySelector('.hotkey-keys').textContent") -eq 'Ctrl+Alt+K')
  Check 'e il file la contiene' ((Get-Content (Join-Path $dir 'mission.json') -Raw) -match 'Ctrl\+Alt\+K')
  Check 'senza la nota del riavvio (vale subito)' ((Win "document.querySelectorAll('.settings-note').length") -eq 0)
  Focus-Settings
  $h = [S3]::Find('EosOS Impostazioni')
  Check 'la finestra delle impostazioni ha il primo piano (i tasti vanno a lei)' (Wait-Until { [S3]::GetForegroundWindow() -eq [IntPtr]$h } 3)
  if ([S3]::GetForegroundWindow() -eq [IntPtr]$h) {
    [S3]::Combo(0x4B);
    Check 'DAL VIVO: la nuova combinazione Ctrl+Alt+K apre Mission Control' (Wait-Until { (Mc-Phase) -ne 'closed' } 5)
    Close-Mc
    Focus-Settings
  }
  Write-Host 'E. conflitti e ritorno alla predefinita'
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'mc.hotkey',value:'Alt+Space'})"
  Check 'la combinazione del Launcher (Alt+Spazio) viene rifiutata come gia usata' (($r.ok -eq $false) -and ($r.error -like '*usata*')) $r.error
  Check 'e Ctrl+Alt+K resta quella attiva' ((Val 'mc.hotkey') -eq 'Ctrl+Alt+K')
  $r = Inv "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'mc.hotkey',value:'Alt+F4'})"
  Check 'Alt+F4 rifiutata' ($r.ok -eq $false)
  Win "Array.from(document.querySelectorAll('.hotkey .settings-button')).find(function(b){return b.textContent==='Cambia'}).click()" | Out-Null; Start-Sleep -Milliseconds 300
  Press "key:' ',code:'Space',altKey:true"; Start-Sleep -Milliseconds 800
  Check 'registrando Alt+Spazio dalla pagina compare il messaggio e resta Ctrl+Alt+K' (((Win "document.querySelector('.settings-error').textContent") -like '*usata*') -and ((Val 'mc.hotkey') -eq 'Ctrl+Alt+K'))
  Win "Array.from(document.querySelectorAll('.hotkey .settings-button')).find(function(b){return b.textContent==='Cambia'}).click()" | Out-Null; Start-Sleep -Milliseconds 300
  Press "key:'ArrowUp',code:'ArrowUp',ctrlKey:true,altKey:true"
  Check 'si torna alla predefinita Ctrl+Alt+Su' (Wait-Until { (Val 'mc.hotkey') -eq 'Ctrl+Alt+Up' } 4)
  Check 'e il file non la conserva piu (e la predefinita)' (-not ((Get-Content (Join-Path $dir 'mission.json') -Raw) -match 'Ctrl\+Alt\+K'))
  Focus-Settings
  if ([S3]::GetForegroundWindow() -eq [IntPtr]([S3]::Find('EosOS Impostazioni'))) {
    [S3]::Combo(0x26)
    Check 'DAL VIVO: Ctrl+Alt+Su apre di nuovo Mission Control' (Wait-Until { (Mc-Phase) -ne 'closed' } 5)
    Close-Mc
  }

  Write-Host 'F. il selettore di cartelle di Windows si apre davvero (app riavviata senza il selettore finto)'
  Cleanup-Eos; Start-Sleep 2
  Remove-Item env:EOSOS_TEST_PICK_FOLDER -ErrorAction SilentlyContinue
  Start-Eos
  Focus-Settings   # la finestra delle impostazioni e il "proprietario" del selettore: deve essere visibile
  [void](Inv "(function(){window.__pick='attesa';window.__TAURI_INTERNALS__.invoke('settings_pick_folder').then(function(r){window.__pick=r});return 1})()")
  Check 'compare la finestra del selettore' (Wait-Until { [S3]::Find('Scegli una cartella') -ne 0 } 8)
  $d = [S3]::Find('Scegli una cartella')
  if ($d -ne 0) { [void][S3]::PostMessage([IntPtr]$d, 0x10, [IntPtr]::Zero, [IntPtr]::Zero) }
  Check 'annullandolo (chiusura) il comando risponde senza cartella' (Wait-Until { (Inv "String(window.__pick)") -eq 'null' } 6) ("risposta " + (Inv "String(window.__pick)"))
  Check 'e EosOS e ancora vivo' (@(Get-Process eosos -ErrorAction SilentlyContinue).Count -ge 1)
}
finally {
  Remove-Item env:EOSOS_TEST_PICK_FOLDER -ErrorAction SilentlyContinue
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
