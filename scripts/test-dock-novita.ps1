# Novita del Dock (10 nov 2026): niente suggerimenti nativi sulle icone (etichetta propria al posto), sei dimensioni, animazione all'apertura di un'app
# (rimbalzo / pulsazione / nessuna), ingrandimento acceso o spento, e l'icona delle APP IN BACKGROUND (area di notifica) con il suo pannello. Non si
# clicca su nessuna app vera: del pannello si guarda l'elenco e si prova un id inventato. mission.json e' salvato e ripristinato.
#
# Uso: powershell -File scripts\test-dock-novita.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class DN { public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern bool GetClientRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern long GetWindowLongPtr(IntPtr h, int i);
 [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int n);
 public static IntPtr Find(string title){ IntPtr f=IntPtr.Zero; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(sb.ToString()==title){f=h;return false;} return true;},IntPtr.Zero); return f; }
 public static string Frame(IntPtr h){ R w,c; GetWindowRect(h,out w); GetClientRect(h,out c); long st=GetWindowLongPtr(h,-16); int pol=-1; DwmGetWindowAttribute(h,2,out pol,4); return (w.Rt-w.L)+"x"+(w.B-w.T)+"|"+c.Rt+"x"+c.B+"|"+(st & 0xC00000).ToString()+"|"+pol; }
 public static string[] Framed(uint pid){ var l=new System.Collections.Generic.List<string>(); EnumWindows((h,x)=>{ uint p; GetWindowThreadProcessId(h,out p); if(p==pid){ long st=GetWindowLongPtr(h,-16); if((st & 0xCB0000)!=0){ var sb=new StringBuilder(120); GetWindowText(h,sb,120); l.Add(sb.ToString()+" stile 0x"+st.ToString("X")); } } return true; },IntPtr.Zero); return l.ToArray(); }
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
 public static bool Visible(string part){ bool f=false; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString().Contains(part)){f=true;return false;} return true;},IntPtr.Zero); return f; } }
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 250 }; $false }
function Set-S([string]$key, [string]$jsValue) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$key',value:$jsValue})" }
function Entry([string]$key) { (@(Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all')" | ForEach-Object { $_ }) | Where-Object { $_.key -eq $key }) }
function Attr([string]$name) { Js 'dock' "document.documentElement.getAttribute('$name')" }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
# Le impostazioni del Dock dell'utente (nascondi automaticamente, dimensione, pile...) falserebbero le misure: la prova parte da quelle di base e il suo file si ripristina.
$isoDir = Join-Path $env:LOCALAPPDATA 'EosOS'
$isoBackup = @{}
foreach ($isoF in 'dock.json', 'mission.json') { $isoP = Join-Path $isoDir $isoF; $isoBackup[$isoF] = if (Test-Path $isoP) { [System.IO.File]::ReadAllBytes($isoP) } else { $null } }
try {
  foreach ($isoF in 'dock.json', 'mission.json') { Remove-Item (Join-Path $isoDir $isoF) -ErrorAction SilentlyContinue }
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 13

  Write-Host 'A. niente suggerimenti nativi (era il rettangolo «EosOS Dock»)'
  Check 'nessuna icona del Dock ha l attributo title' ((Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item[title]').length") -eq 0)
  Check 'le icone hanno la loro etichetta (data-label)' ((Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-item[data-label]').length") -ge 2)
  Check 'l icona del Launchpad si chiama Launchpad' ((Js 'dock' "document.querySelector('.dock-launchpad').getAttribute('data-label')") -eq 'Launchpad')

  Write-Host 'A2. nessuna cornice (era la barra del titolo «EosOS Dock» in cima al Dock)'
  $fr = [DN]::Frame([DN]::Find('EosOS Dock'))
  Write-Host ("  (finestra|client|stile con cornice|criterio DWM: " + $fr + ")")
  $parts = $fr -split '\|'
  Check 'la finestra del Dock non ha lo stile con barra del titolo (WS_CAPTION)' ($parts[2] -eq '0')
  Check 'e la sua area client e grande quanto la finestra (nessuna parte non client)' ($parts[0] -eq $parts[1])

  $mainPidDN = [uint32](Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notmatch '--keyhook|--watchdog' } | Select-Object -First 1).ProcessId
  Start-Sleep -Seconds 1
  $framed = @([DN]::Framed($mainPidDN))
  Check 'nessuna finestra di EosOS (barre, pannello, Launcher, Impostazioni...) ha la barra del titolo di Windows' ($framed.Count -eq 0) ($framed -join ' ; ')

  Write-Host 'B. dimensioni'
  $size = Entry 'dock.size'
  $opts = @($size.kind.options | ForEach-Object { $_[0] })
  Check 'le dimensioni sono sei' ($opts.Count -eq 6) ($opts -join ',')
  Check 'ci sono la piu piccola e la piu grande' (($opts -contains 'tiny') -and ($opts -contains 'xxl'))
  foreach ($sz in 'tiny', 'xxl', 'medium') {
    $r = Set-S 'dock.size' "'$sz'"
    Check "scegliere $sz" ($r.ok -eq $true)
    Start-Sleep -Milliseconds 700
    $icon = [double](Js 'dock' "getComputedStyle(document.documentElement).getPropertyValue('--dock-icon').replace('px','')")
    $exp = @{ tiny = 33; medium = 50; xxl = 80 }[$sz]
    Check "con $sz l icona e circa $exp px" ([math]::Abs($icon - $exp) -le 3) "icona $icon px"
  }

  Write-Host 'C. animazione e ingrandimento'
  Check 'di base e il rimbalzo' ((Attr 'data-dock-anim') -eq 'bounce')
  foreach ($a in 'pulse', 'none', 'bounce') {
    $r = Set-S 'dock.anim' "'$a'"
    Check "animazione ${a}: scelta e applicata subito" (($r.ok -eq $true) -and (Wait-Until { (Attr 'data-dock-anim') -eq $a } 4))
  }
  Check 'una scelta inventata e rifiutata' (((Set-S 'dock.anim' "'saltello'").ok) -eq $false)
  Check 'l ingrandimento e acceso di base' ((Js 'dock' "window.__TAURI_INTERNALS__.invoke('dock_settings').then(function(s){return s.magnify})") -eq $true)
  [void](Set-S 'dock.magnify' 'false')
  Check 'spento dalle Impostazioni arriva alla pagina del Dock' (Wait-Until { (Js 'dock' "window.__TAURI_INTERNALS__.invoke('dock_settings').then(function(s){return s.magnify})") -eq $false } 4)
  [void](Set-S 'dock.magnify' 'true')

  Write-Host 'C2. ingrandimento: con il mouse fermo non cambia niente'
  Check 'le icone non hanno --s a riposo' ((Js 'dock' "Array.from(document.querySelectorAll('.dock:not(.dock-ghost) .dock-item')).every(function(c){return c.style.getPropertyValue('--s')===''})") -eq $true)

  Write-Host 'D. app in background'
  $cnt = [int](Js 'dock' "window.__TAURI_INTERNALS__.invoke('dock_tray_count')")
  Write-Host ("  (app in background trovate: " + $cnt + ")")
  if ($cnt -gt 0) {
    Check 'l icona compare nel Dock' (Wait-Until { (Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-tray').length") -eq 1 } 12)
    Check 'ha un nome per i lettori di schermo' ((Js 'dock' "document.querySelector('.dock-tray').getAttribute('aria-label')") -like 'App in background*')
    $list = @(Js 'panel' "window.__TAURI_INTERNALS__.invoke('tray_apps')" | ForEach-Object { $_ })
    Check 'l elenco del pannello ha almeno un app con nome e id' (($list.Count -ge 1) -and (-not [string]::IsNullOrEmpty($list[0].name)) -and (-not [string]::IsNullOrEmpty($list[0].id))) ("app: " + (($list | ForEach-Object { $_.name }) -join ', '))
    Js 'dock' "document.querySelector('.dock-tray').click()" | Out-Null
    Check 'un clic apre il pannello' (Wait-Until { [DN]::Visible('EosOS Pannello') -and ((Js 'panel' "document.querySelectorAll('.tray-app').length") -ge 1) } 8)
    Check 'il pannello mostra le stesse app' ((Js 'panel' "document.querySelectorAll('.tray-app').length") -eq $list.Count) ("pannello " + (Js 'panel' "document.querySelectorAll('.tray-app').length") + " / elenco " + $list.Count)
    Check 'ogni app ha il nome' ((Js 'panel' "Array.from(document.querySelectorAll('.tray-app')).every(function(b){return (b.getAttribute('aria-label')||'').length>0})") -eq $true)
    Check 'un id inventato non apre niente' ((Js 'panel' "window.__TAURI_INTERNALS__.invoke('tray_open',{id:'c:\\\\non\\\\esiste.exe'})") -eq $false)
    [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Start-Sleep 1
    [void](Set-S 'dock.tray' 'false')
    Check 'spento dalle Impostazioni l icona sparisce' (Wait-Until { (Js 'dock' "document.querySelectorAll('.dock:not(.dock-ghost) .dock-tray').length") -eq 0 } 12)
    [void](Set-S 'dock.tray' 'true')
  } else { Write-Host '  (nessuna app in background su questo PC adesso: salto le prove dell icona)' }

  Write-Host 'E. controllo accessi'
  Cleanup-Eos; Start-Sleep 1
  $env:EOSOS_TEST_STRICT = '1'
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  Check 'dalla barra dock_tray_count da 0' (([int](Js 'bar' "window.__TAURI_INTERNALS__.invoke('dock_tray_count').then(function(r){return r},function(e){return 0})")) -eq 0)
  Check 'dal Dock tray_apps non da niente' ((Js 'dock' "window.__TAURI_INTERNALS__.invoke('tray_apps').then(function(r){return r.length},function(e){return 0})") -eq 0)
  Check 'dal Dock tray_open non apre' ((Js 'dock' "window.__TAURI_INTERNALS__.invoke('tray_open',{id:'x'}).then(function(r){return r},function(e){return false})") -ne $true)
}
finally {
  foreach ($isoF in 'dock.json', 'mission.json') { $isoP = Join-Path $isoDir $isoF; Remove-Item $isoP -ErrorAction SilentlyContinue; if ($null -ne $isoBackup[$isoF]) { [System.IO.File]::WriteAllBytes($isoP, $isoBackup[$isoF]) } }
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
