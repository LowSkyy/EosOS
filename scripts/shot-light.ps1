# Foto delle pagine di EosOS con il tema CHIARO forzato (e scuro per confronto), per controllare a occhio contrasto e tinte.
# Si forza solo `data-theme` nella pagina (niente cambia nel sistema). Salva i PNG in -Out (una cartella nuova).
# Uso: powershell -File scripts\shot-light.ps1 -Out C:\percorso\foto [-Exe percorso\eosos.exe]
param([string]$Out, [string]$Exe)
$ErrorActionPreference = 'Stop'
if (-not $Exe) { $Exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
New-Item -ItemType Directory -Force $Out | Out-Null
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
function Shot([string]$view, [string]$file, [string]$pre) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; & node $cdp shot (Join-Path $Out $file) $pre 2>$null | Out-Null }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bk = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $Exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 14
  # Apre il pannello dell'orologio e le Impostazioni (per averne le pagine vive).
  [void](Js 'bar' "document.querySelector('.clock').click()"); Start-Sleep 2
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:'dock'})"); Start-Sleep 2
  foreach ($theme in 'light', 'dark') {
    $set = "document.documentElement.setAttribute('data-theme','$theme')"
    Shot 'bar' "bar-$theme.png" $set
    Shot 'dock' "dock-$theme.png" $set
    Shot 'panel' "panel-$theme.png" $set
    Shot 'settings' "settings-$theme.png" $set
  }
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
}
finally {
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bk) { [System.IO.File]::WriteAllBytes($mc, $bk) }
}
Get-ChildItem $Out | ForEach-Object { Write-Host $_.Name }
