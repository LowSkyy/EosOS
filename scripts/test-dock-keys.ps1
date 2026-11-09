# Scorciatoie per le app del Dock (di base Ctrl+Alt+1..9): le combinazioni risultano davvero registrate (un altro processo non
# riesce a registrarle: errore 1409 «gia' registrata»), si spengono e si cambiano dalle Impostazioni, e «premere» il numero 1 apre
# la prima app del Dock. Non si preme nessun tasto: la pressione e' simulata dal comando di prova. Per non toccare le tue app il
# Dock viene limitato per la prova a UN'app mia (Blocco note, solo quelli nuovi si chiudono) e dock.json e mission.json vengono
# salvati per intero e ripristinati.
#
# Uso: powershell -File scripts\test-dock-keys.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices;
public static class HK {
  [DllImport("user32.dll", SetLastError=true)] static extern bool RegisterHotKey(IntPtr h, int id, uint mods, uint vk);
  [DllImport("user32.dll")] static extern bool UnregisterHotKey(IntPtr h, int id);
  // true se la combinazione era LIBERA (e la rilascia subito); false con codice 1409 se era gia' presa.
  public static string Probe(uint mods, uint vk) {
    bool ok = RegisterHotKey(IntPtr.Zero, 0x7A01, mods | 0x4000, vk);
    int err = Marshal.GetLastWin32Error();
    if (ok) { UnregisterHotKey(IntPtr.Zero, 0x7A01); return "libera"; }
    return "presa:" + err;
  }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Registered { [int](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('dockkeys_registered')") }
function Set-S([string]$k, [string]$v) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$k',value:$v})" }
function Start-Eos([hashtable]$extra = @{}) {
  Cleanup-Eos; Start-Sleep 1
  foreach ($k in $extra.Keys) { Set-Item "env:$k" $extra[$k] }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
}
# Modificatori: 1 Alt, 2 Ctrl, 4 Maiusc, 8 Windows.
$CTRL_ALT = 3; $CTRL_SHIFT = 6; $ALT_SHIFT = 5
function Probe-All([uint32]$mods) { @(1..9 | ForEach-Object { [HK]::Probe($mods, 0x30 + $_) }) }

$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkDj = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$notepadBefore = @(Get-Process notepad -ErrorAction SilentlyContinue | ForEach-Object { $_.Id })
try {
  Remove-Item $mc -ErrorAction SilentlyContinue
  Write-Host 'A. di base nelle build di prova: spente'
  Start-Eos
  Check 'nessuna scorciatoia registrata' ((Registered) -eq 0)
  Check 'le combinazioni sono libere' ((@(Probe-All $CTRL_ALT | Where-Object { $_ -ne 'libera' })).Count -eq 0)
  Check 'l impostazione dice Ctrl+Alt (la scelta di base)' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.keys'})[0].value})") -eq 'ctrl_alt')

  Write-Host 'B. accese (EOSOS_DOCK_KEYS=1, come in produzione)'
  Start-Eos @{ EOSOS_DOCK_KEYS = '1' }
  Check 'nove scorciatoie registrate' (Wait-Until { (Registered) -eq 9 } 6) ("registrate: " + (Registered))
  $p = Probe-All $CTRL_ALT
  Check 'un altro processo non riesce a registrarle (errore 1409: gia presa)' ((@($p | Where-Object { $_ -ne 'presa:1409' })).Count -eq 0) ($p -join ' ')
  Check 'le altre combinazioni restano libere (Alt+Maiusc+1..9)' ((@(Probe-All $ALT_SHIFT | Where-Object { $_ -ne 'libera' })).Count -eq 0)

  Write-Host 'C. cambiare modificatore e spegnere'
  $r = Set-S 'dock.keys' "'alt_shift'"
  Check 'la scelta si salva' ($r.ok -eq $true)
  Check 'ora sono prese le combinazioni Alt+Maiusc e libere quelle Ctrl+Alt' (Wait-Until { ((@(Probe-All $ALT_SHIFT | Where-Object { $_ -ne 'presa:1409' })).Count -eq 0) -and ((@(Probe-All $CTRL_ALT | Where-Object { $_ -ne 'libera' })).Count -eq 0) } 6)
  [void](Set-S 'dock.keys' "'off'")
  Check 'spente: nessuna registrata e tutte libere' ((Wait-Until { (Registered) -eq 0 } 6) -and ((@(Probe-All $ALT_SHIFT | Where-Object { $_ -ne 'libera' })).Count -eq 0))
  Check 'una scelta inventata e rifiutata' (((Set-S 'dock.keys' "'tutti'").ok) -eq $false)
  [void](Set-S 'dock.keys' "'ctrl_shift'")
  Check 'Ctrl+Maiusc+1..9 funziona' ((Wait-Until { (Registered) -eq 9 } 6) -and ((@(Probe-All $CTRL_SHIFT | Where-Object { $_ -ne 'presa:1409' })).Count -eq 0))

  Write-Host 'D. «premere» il numero 1 apre la prima app (un Blocco note mio)'
  Check 'un numero senza app non fa niente' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('dockkeys_fire',{n:99})") -eq $false)
  $np = Join-Path $env:WINDIR 'System32\notepad.exe'
  $json = '{ "version": 1, "pinned": [ { "name": "Blocco note", "path": "' + ($np -replace '\\', '\\') + '" } ], "autohide": false, "glass": 0.7, "blur": true, "reserve": true }'
  [System.IO.File]::WriteAllText($dj, $json, (New-Object System.Text.UTF8Encoding($false)))
  Start-Sleep 2
  Check 'il Dock ha come prima app il Blocco note' (Wait-Until { (Js 'dock' "(document.querySelector('.dock:not(.dock-ghost) .dock-item:not(.dock-stack):not(.dock-launchpad)')||{getAttribute:function(){return ''}}).getAttribute('data-label')") -eq 'Blocco note' } 8)
  $r = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('dockkeys_fire',{n:1})"
  Check 'la pressione simulata del numero 1 e accettata' ($r -eq $true)
  Check 'parte un nuovo Blocco note' (Wait-Until { @(Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $notepadBefore -notcontains $_.Id }).Count -ge 1 } 12)
}
finally {
  Cleanup-Eos
  Remove-Item env:EOSOS_DOCK_KEYS -ErrorAction SilentlyContinue
  # Si chiudono SOLO i Blocco note nati durante la prova.
  Get-Process notepad -ErrorAction SilentlyContinue | Where-Object { $notepadBefore -notcontains $_.Id } | Stop-Process -Force -ErrorAction SilentlyContinue
  if ($null -ne $bkDj) { [System.IO.File]::WriteAllBytes($dj, $bkDj) }
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
