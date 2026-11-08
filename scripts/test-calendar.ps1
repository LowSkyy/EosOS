# Verifica N2 del Centro notifiche: il calendario sotto l'orologio. Un clic sull'orologio lo apre, mostra il mese di
# oggi (confrontato con le date del sistema, comprese le settimane ISO calcolate da .NET), si cambia mese con i pulsanti
# e con le frecce, «Oggi» riporta al mese corrente, un secondo clic o Esc lo chiudono. Nessuna impostazione toccata,
# nessuna finestra di altri toccata. L'unico tasto vero e Esc, dopo aver verificato che il pannello sia in primo piano.
#
# Uso: powershell -File scripts\test-calendar.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class CA {
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  public static string Title(IntPtr h) { var sb = new StringBuilder(512); GetWindowText(h, sb, 512); return sb.ToString(); }
  public static long Find(string part) { long f = 0; EnumWindows((h, l) => { if (IsWindowVisible(h) && Title(h).Contains(part)) { f = h.ToInt64(); return false; } return true; }, IntPtr.Zero); return f; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Cal([string]$expr) { Js 'panel' $expr }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 120 }; $false }
function Panel-Open { [CA]::Find('EosOS Pannello') -ne 0 }
function Title { Cal "document.querySelector('.cal-title').textContent" }
function Click-Btn([string]$label) { Cal "(Array.from(document.querySelectorAll('button')).find(function(b){return b.getAttribute('aria-label')==='$label'})||{click:function(){}}).click()" | Out-Null; Start-Sleep -Milliseconds 300 }
function Key([string]$k) { Cal "window.dispatchEvent(new KeyboardEvent('keydown',{key:'$k',bubbles:true,cancelable:true}))" | Out-Null; Start-Sleep -Milliseconds 300 }
$mesi = 'gennaio', 'febbraio', 'marzo', 'aprile', 'maggio', 'giugno', 'luglio', 'agosto', 'settembre', 'ottobre', 'novembre', 'dicembre'
function Mese([datetime]$d) { $mesi[$d.Month - 1] + ' ' + $d.Year }

try {
  Cleanup-Eos; Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS; Start-Sleep 12
  $now = Get-Date

  Write-Host 'A. apertura dall orologio'
  Check 'l orologio e un pulsante con il titolo Calendario' ((Js 'bar' "(function(){var c=document.querySelector('.clock');return c.tagName+'|'+c.title})()") -eq 'BUTTON|Calendario')
  Check 'all inizio il calendario non si vede' (-not (Panel-Open))
  Js 'bar' "document.querySelector('.clock').click()" | Out-Null
  Check 'un clic sull orologio apre il pannello' (Wait-Until { (Panel-Open) -and ((Title) -ne $null) } 6)
  Check 'mostra il mese di oggi' ((Title) -eq (Mese $now)) ((Title) + ' / atteso ' + (Mese $now))

  Write-Host 'B. il mese mostrato'
  Check 'intestazione: settimana, lun mar mer gio ven sab dom' (((Cal "Array.from(document.querySelectorAll('.cal-grid thead th')).map(function(e){return e.textContent.trim()}).join('|')") -replace '^\|', '|') -eq '|lun|mar|mer|gio|ven|sab|dom')
  Check 'sei righe di sette giorni' (((Cal "document.querySelectorAll('.cal-grid tbody tr').length") -eq 6) -and ((Cal "document.querySelectorAll('.cal-day').length") -eq 42))
  $today = Cal "(function(){var t=document.querySelectorAll('.cal-today');return t.length+'|'+(t[0]?t[0].textContent:'')+'|'+(t[0]?t[0].getAttribute('aria-current'):'')})()"
  Check 'un solo giorno evidenziato: oggi' ($today -eq ('1|' + $now.Day + '|date')) $today
  $first = [datetime]::new($now.Year, $now.Month, 1)
  $offset = (([int]$first.DayOfWeek + 6) % 7)   # lunedi = 0
  $firstCell = Cal "(function(){var c=document.querySelectorAll('.cal-day');var i=Array.from(c).findIndex(function(e){return !e.classList.contains('cal-out')});return i})()"
  Check 'il primo del mese cade nella colonna giusta (settimana da lunedi)' ($firstCell -eq $offset) "colonna $firstCell, attesa $offset"
  $days = Cal "Array.from(document.querySelectorAll('.cal-day:not(.cal-out)')).length"
  Check 'i giorni del mese sono quelli giusti' ($days -eq [datetime]::DaysInMonth($now.Year, $now.Month)) "giorni $days"
  $weeks = @(Cal "Array.from(document.querySelectorAll('.cal-grid tbody th')).map(function(e){return Number(e.textContent)})" | ForEach-Object { $_ })
  $thu = $first.AddDays(-$offset + 3)   # il giovedi della prima riga decide la settimana ISO (calcolo di .NET Framework, indipendente da quello della pagina)
  $expectedFirstWeek = [System.Globalization.CultureInfo]::InvariantCulture.Calendar.GetWeekOfYear($thu, [System.Globalization.CalendarWeekRule]::FirstFourDayWeek, [DayOfWeek]::Monday)
  Check 'i numeri di settimana sono quelli ISO (calcolati da .NET Framework)' ($weeks[0] -eq $expectedFirstWeek -and ($weeks[5] - $weeks[0] -eq 5 -or $weeks[0] -gt 40)) ("settimane " + ($weeks -join ',') + ", attesa la prima " + $expectedFirstWeek)
  Check 'ogni giorno ha una descrizione completa per i lettori di schermo' ((Cal "document.querySelector('.cal-today').getAttribute('aria-label')") -like ('*' + $now.Day + ' ' + $mesi[$now.Month - 1] + ' ' + $now.Year))

  Write-Host 'C. cambiare mese'
  Click-Btn 'Mese successivo'
  Check 'il pulsante avanti mostra il mese dopo' ((Title) -eq (Mese $now.AddMonths(1))) (Title)
  Check 'e in un altro mese non c e nessun oggi' ((Cal "document.querySelectorAll('.cal-today').length") -eq 0)
  Click-Btn 'Mese precedente'; Click-Btn 'Mese precedente'
  Check 'due volte indietro: il mese prima di oggi' ((Title) -eq (Mese $now.AddMonths(-1))) (Title)
  Check 'il pulsante Oggi e attivo' ((Cal "!document.querySelector('.cal-today-btn').disabled") -eq $true)
  Cal "document.querySelector('.cal-today-btn').click()" | Out-Null; Start-Sleep -Milliseconds 300
  Check 'Oggi torna al mese corrente' ((Title) -eq (Mese $now))
  Check 'e il pulsante Oggi si disattiva' ((Cal "document.querySelector('.cal-today-btn').disabled") -eq $true)
  Key 'ArrowRight'; Key 'ArrowRight'
  Check 'freccia destra due volte: due mesi avanti' ((Title) -eq (Mese $now.AddMonths(2))) (Title)
  Key 'PageUp'
  Check 'PagSu: un mese indietro' ((Title) -eq (Mese $now.AddMonths(1))) (Title)
  Key 'Home'
  Check 'Home: torna a oggi' ((Title) -eq (Mese $now))
  1..14 | ForEach-Object { Key 'ArrowRight' }
  Check 'attraversando l anno il mese e giusto (14 mesi dopo)' ((Title) -eq (Mese $now.AddMonths(14))) (Title)

  Write-Host 'D. chiusura'
  Key 'Home'
  Js 'bar' "document.querySelector('.clock').click()" | Out-Null
  Check 'un secondo clic sull orologio chiude' (Wait-Until { -not (Panel-Open) } 4)
  Js 'bar' "document.querySelector('.clock').click()" | Out-Null; Start-Sleep -Milliseconds 800
  Check 'riaperto riparte dal mese di oggi' ((Panel-Open) -and ((Title) -eq (Mese $now)))
  $h = [CA]::Find('EosOS Pannello')
  if (Wait-Until { [CA]::GetForegroundWindow() -eq [IntPtr]$h } 3) {
    [CA]::keybd_event(0x1B, 0, 0, [UIntPtr]::Zero); [CA]::keybd_event(0x1B, 0, 2, [UIntPtr]::Zero)
    Check 'Esc con la tastiera vera lo chiude' (Wait-Until { -not (Panel-Open) } 4)
  } else { Check 'il pannello ha il primo piano per provare Esc' $false }
  Check 'gli indicatori hanno ancora il loro pannello (stesso pannello, nessun conflitto)' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_toggle',{key:'volume',left:100,top:4,width:30,height:20}).then(function(){return 'ok'})") -eq 'ok')
  Start-Sleep -Milliseconds 800
  Check 'e mostra il volume, non il calendario' ((Cal "document.querySelectorAll('.calendar').length") -eq 0)
}
finally {
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')")
  Cleanup-Eos
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
