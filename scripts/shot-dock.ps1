# Foto dello schermo intorno al Dock (per guardare a occhio bordi e icone). Con la sfocatura attiva la finestra del Dock e'
# esclusa dalle catture: lo script la spegne un momento e poi la rimette com'era. Non tocca nient'altro.
# Uso: powershell -File scripts\shot-dock.ps1 -Out C:\percorso\dock.png [-Exe percorso\eosos.exe] [-Pre "JS da eseguire nel Dock prima"]
param([string]$Out, [string]$Exe, [string]$Pre = '')
$ErrorActionPreference = 'Stop'
if (-not $Exe) { $Exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
$alreadyRunning = @(Get-Process eosos -ErrorAction SilentlyContinue).Count -gt 0
if (-not $alreadyRunning) {
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $Exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 12
}
$blur = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='dock.blur'})[0].value})"
if ($blur) { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:false})"); Start-Sleep 2 }
if ($Pre) { [void](Js 'dock' $Pre); Start-Sleep 1 }
$scr = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$w = 1100; $h = 190
$x = $scr.Left + [int](($scr.Width - $w) / 2); $y = $scr.Bottom - $h
$bmp = New-Object System.Drawing.Bitmap $w, $h
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($x, $y, 0, 0, (New-Object System.Drawing.Size $w, $h))
$bmp.Save($Out, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
if ($blur) { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:true})") }
if (-not $alreadyRunning) { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
Write-Host "Salvata: $Out"
