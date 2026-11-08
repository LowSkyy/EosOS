# Centro notifiche, fase N6: rifiniture del pannello sotto l'orologio. (1) La scelta «La settimana comincia da»
# (lunedi o domenica) dalle Impostazioni: intestazione, prima colonna, weekend e numeri di settimana ISO confrontati con
# .NET. (2) Accessibilita e tastiera del pannello: ogni pulsante ha un nome, la tabella ha le intestazioni, il focus ha un
# contorno. mission.json viene salvato e ripristinato. Nessuna notifica viene letta (la lettura resta spenta).
#
# Uso: powershell -File scripts\test-calendar-n6.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class N6 {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); if (IsWindowVisible(h) && sb.ToString().Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Pn([string]$expr) { Js 'panel' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 150 }; $false }
function Panel-Open { [N6]::Find('EosOS Pannello') -ne 0 }
function Open-Panel { Js 'bar' "document.querySelector('.clock').click()" | Out-Null; Wait-Until { (Panel-Open) -and ((Pn "document.querySelectorAll('.cal-day').length") -eq 42) } 6 }
function Close-Panel { [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')"); Wait-Until { -not (Panel-Open) } 3 | Out-Null; Start-Sleep -Milliseconds 300 }

$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$backup = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1
  Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $now = Get-Date
  $first = [datetime]::new($now.Year, $now.Month, 1)
  function Heads { (Pn "Array.from(document.querySelectorAll('.cal-grid thead th')).map(function(e){return e.textContent.trim()}).join('|')") }

  Write-Host 'A. di default la settimana comincia da lunedi'
  Check 'la scelta nelle Impostazioni esiste ed e Lunedi' (((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='calendar.week_start'})[0].value})")) -eq 'mon')
  Check 'il pannello si apre' (Open-Panel)
  Check 'intestazione da lunedi' ((Heads) -eq '|lun|mar|mer|gio|ven|sab|dom')

  Write-Host 'B. domenica'
  $r = Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'calendar.week_start',value:'sun'})"
  Check 'la scelta si salva e vale subito' (($r.ok -eq $true) -and ($r.live -eq $true))
  Close-Panel; Check 'riaperto il pannello' (Open-Panel)
  Check 'intestazione da domenica' ((Heads) -eq '|dom|lun|mar|mer|gio|ven|sab')
  $offset = [int]$first.DayOfWeek   # domenica = 0
  $firstCell = Pn "Array.from(document.querySelectorAll('.cal-day')).findIndex(function(e){return !e.classList.contains('cal-out')})"
  Check 'il primo del mese cade nella colonna giusta' ($firstCell -eq $offset) "colonna $firstCell, attesa $offset"
  Check 'weekend: prima e ultima colonna' ((Pn "(function(){var r=document.querySelectorAll('.cal-grid tbody tr')[1];var td=r.querySelectorAll('.cal-day');return td[0].classList.contains('cal-weekend')&&td[6].classList.contains('cal-weekend')&&!td[3].classList.contains('cal-weekend')})()") -eq $true)
  $thu = $first.AddDays(-$offset + 4)   # il giovedi della prima riga (domenica = colonna 0, giovedi = colonna 4)
  $expected = [System.Globalization.CultureInfo]::InvariantCulture.Calendar.GetWeekOfYear($thu, [System.Globalization.CalendarWeekRule]::FirstFourDayWeek, [DayOfWeek]::Monday)
  $w0 = Pn "Number(document.querySelector('.cal-grid tbody th').textContent)"
  Check 'il numero di settimana della prima riga e quello ISO calcolato da .NET' ($w0 -eq $expected) "pagina $w0, .NET $expected"
  Check 'un solo oggi, e con il numero giusto' ((Pn "(function(){var t=document.querySelectorAll('.cal-today');return t.length+'|'+t[0].textContent})()") -eq ('1|' + $now.Day))
  Check 'il giorno di oggi cade nella colonna giusta' ((Pn "Array.from(document.querySelectorAll('.cal-grid tbody td')).findIndex(function(e){return e.classList.contains('cal-today')}) % 7") -eq ([int]$now.DayOfWeek))

  Write-Host 'C. di nuovo lunedi'
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'calendar.week_start',value:'mon'})")
  Close-Panel; [void](Open-Panel)
  Check 'torna l intestazione da lunedi' ((Heads) -eq '|lun|mar|mer|gio|ven|sab|dom')
  Check 'un valore inventato e rifiutato e non cambia nulla' (((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'calendar.week_start',value:'mercoledi'})").ok) -eq $false)

  Write-Host 'D. accessibilita del pannello'
  $bad = Pn "(function(){var out=[];document.querySelectorAll('.calendar button, .calendar a, .calendar input').forEach(function(el){var n=el.getAttribute('aria-label')||(el.textContent||'').trim();if(!n)out.push(el.className)});return out.join(',')})()"
  Check 'ogni pulsante ha un nome' ([string]::IsNullOrEmpty($bad)) $bad
  Check 'la tabella ha il titolo e le intestazioni di colonna e di riga' (((Pn "document.querySelector('.cal-grid').getAttribute('aria-label')") -like '* 20*') -and ((Pn "document.querySelectorAll('.cal-grid thead th[scope=col]').length") -eq 8) -and ((Pn "document.querySelectorAll('.cal-grid tbody th[scope=row]').length") -eq 6))
  Check 'ogni giorno ha una descrizione completa' ((Pn "Array.from(document.querySelectorAll('.cal-day')).every(function(e){return /\d{4}$/.test(e.getAttribute('aria-label'))})") -eq $true)
  Check 'il gruppo del pannello ha un nome' ((Pn "document.querySelector('.calendar').getAttribute('aria-label')") -eq 'Notifiche e calendario')
  Check 'c e un contorno del focus per i pulsanti (stile)' ((Pn "(function(){var s=Array.from(document.styleSheets).flatMap(function(x){try{return Array.from(x.cssRules)}catch(e){return []}}).map(function(r){return r.cssText}).join(' ');return s.indexOf('.cal-btn:focus-visible')>=0&&s.indexOf('.notifs-x:focus-visible')>=0})()") -eq $true)
  Pn "document.querySelector('[aria-label=\'Mese successivo\']')" | Out-Null
  Check 'il titolo del mese e annunciato (aria-live)' ((Pn "document.querySelector('.cal-title').getAttribute('aria-live')") -eq 'polite')
}
finally {
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $backup) { [System.IO.File]::WriteAllBytes($mc, $backup) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
