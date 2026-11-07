# Verifica il Launcher (L2) con tastiera vera: Alt+Spazio lo apre sul monitor del cursore, a fuoco;
# si digita con tasti veri; Esc lo chiude; Invio avvia un'app (la Calcolatrice, app di prova);
# la perdita di focus lo nasconde; la stessa scorciatoia lo chiude. Non usa SendKeys e non fa clic.
#
# Uso: powershell -File scripts\test-launcher.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = Resolve-Path $Exe } else { $exe = Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path }
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class LT {
  [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
  public delegate bool EP(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern void keybd_event(byte vk, byte scan, uint flags, UIntPtr extra);
  [DllImport("user32.dll")] public static extern int GetWindowLong(IntPtr h, int i);
  public static IntPtr Find(string title) { IntPtr f = IntPtr.Zero; EnumWindows((h, l) => { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); if (sb.ToString() == title) { f = h; return false; } return true; }, IntPtr.Zero); return f; }
  [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint m, IntPtr w, IntPtr l);
  public static bool PostClose(IntPtr h) { return PostMessage(h, 0x10, IntPtr.Zero, IntPtr.Zero); }
  public static void Key(byte vk) { keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); }
  public static void Ctrl(byte vk) { keybd_event(0x11, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 0, UIntPtr.Zero); keybd_event(vk, 0, 2, UIntPtr.Zero); keybd_event(0x11, 0, 2, UIntPtr.Zero); }
  public static void AltSpace() { keybd_event(0x12, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 0, UIntPtr.Zero); keybd_event(0x20, 0, 2, UIntPtr.Zero); keybd_event(0x12, 0, 2, UIntPtr.Zero); }
}
'@
$script:fail = 0
function Check($name, $ok, $detail = '') {
  if ($ok) { Write-Host "  OK   $name $detail" } else { Write-Host "  FAIL $name $detail" -ForegroundColor Red; $script:fail++ }
}
function Js($expr) { $env:CDP_VIEW = 'launcher'; $o = node (Join-Path $PSScriptRoot 'cdp-dock.mjs') eval $expr; if ($o) { $o | ConvertFrom-Json } }
function Launcher { [LT]::Find('EosOS Launcher') }
function IsOpen { $h = Launcher; $h -ne [IntPtr]::Zero -and [LT]::IsWindowVisible($h) }
# I tasti veri vanno SOLO al Launcher: se non e in primo piano si salta, per non scrivere in una finestra dell'utente.
function Focused { $h = Launcher; $h -ne [IntPtr]::Zero -and [LT]::GetForegroundWindow() -eq $h }
function Press($vk) { if (Focused) { [LT]::Key([byte]$vk) } else { Write-Host '  (tasto saltato: il Launcher non e in primo piano)' -ForegroundColor Yellow } }
function TypeText($text) { foreach ($c in $text.ToCharArray()) { Press ([int][char]$c.ToString().ToUpper()); Start-Sleep -Milliseconds 60 } }
function PressCtrl($vk) { if (Focused) { [LT]::Ctrl([byte]$vk) } else { Write-Host '  (tasto saltato: il Launcher non e in primo piano)' -ForegroundColor Yellow } }
function Cleanup {
  Get-Process | Where-Object { $_.ProcessName -like 'Calculator*' } | ForEach-Object { [void]$_.CloseMainWindow() }
  Start-Sleep 1
  Get-Process | Where-Object { $_.ProcessName -like 'Calculator*' } | Stop-Process -Force -ErrorAction SilentlyContinue
  Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
}
# Le prove di conferma premono Invio su Riavvia/Spegni: SOLO con una build di prova, che le registra
# in %TEMP%\eosos-actions.log invece di eseguirle. Si controlla che l'eseguibile contenga quella protezione.
$bytes = [System.IO.File]::ReadAllBytes($exe)
$text = [System.Text.Encoding]::GetEncoding(28591).GetString($bytes)
if (-not $text.Contains('eosos-actions.log')) { Write-Host 'Questo eseguibile non e una build di prova (feature test-hooks): rifiuto di provare Riavvia/Spegni.'; exit 2 }
$actionLog = Join-Path $env:TEMP 'eosos-actions.log'
Remove-Item $actionLog -ErrorAction SilentlyContinue
$savedClip = $null; try { $savedClip = Get-Clipboard -Raw -ErrorAction Stop } catch {}
$calcBefore = @(Get-Process | Where-Object { $_.ProcessName -like 'Calculator*' }).Count
if ($calcBefore -gt 0) { Write-Host 'La Calcolatrice e gia aperta: chiudila e riprova (non e mia).'; exit 2 }
$env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
Start-Process $exe | Out-Null
try {
  $t = 0; while ((Launcher) -eq [IntPtr]::Zero -and $t -lt 40) { Start-Sleep -Milliseconds 500; $t++ }
  Start-Sleep 2
  Write-Host 'A. creazione'
  Check 'finestra creata e nascosta' ((Launcher) -ne [IntPtr]::Zero -and -not (IsOpen))
  $ex = [LT]::GetWindowLong((Launcher), -20)
  Check 'non in Alt+Tab (TOOLWINDOW), attivabile (no NOACTIVATE)' ((($ex -band 0x80) -ne 0) -and (($ex -band 0x08000000) -eq 0)) ('ex=0x{0:X}' -f $ex)

  Write-Host 'B. Alt+Spazio apre, a fuoco, centrata'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  Check 'aperta' (IsOpen)
  $h = Launcher; $r = New-Object LT+R; [void][LT]::GetWindowRect($h, [ref]$r)
  Add-Type -AssemblyName System.Windows.Forms
  $w = $r.Rt - $r.L
  $b = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  Check 'larghezza 680 a scala 100%' ($w -eq 680) "w=$w"
  Check 'centrata in orizzontale' ([math]::Abs(($r.L + $r.Rt) / 2 - ($b.X + $b.Width / 2)) -le 2) "L=$($r.L) R=$($r.Rt)"
  Check 'a un quinto dall''alto' ([math]::Abs($r.T - ($b.Y + [math]::Floor($b.Height / 5))) -le 2) "T=$($r.T)"
  Check 'in primo piano' ([LT]::GetForegroundWindow() -eq $h)
  Check 'campo a fuoco nella pagina' ((Js "document.activeElement && document.activeElement.className") -eq 'launcher-input')

  Write-Host 'C. tastiera vera'
  TypeText 'calc'; Start-Sleep -Milliseconds 800
  Check 'testo digitato' ((Js "document.querySelector('.launcher-input').value") -eq 'calc')
  Check 'primo risultato Calcolatrice' ((Js "document.querySelector('.launcher-name') && document.querySelector('.launcher-name').textContent") -eq 'Calcolatrice')
  [void][LT]::GetWindowRect($h, [ref]$r)
  Check 'altezza cresce con i risultati' (($r.B - $r.T) -gt 100) "h=$($r.B - $r.T)"
  Check 'prima riga selezionata' ((Js "document.querySelector('.launcher-item').dataset.selected") -eq 'true')

  Write-Host 'D. Esc chiude'
  Press 0x1B; Start-Sleep -Milliseconds 600
  Check 'chiusa con Esc' (-not (IsOpen))

  Write-Host 'E. riapertura azzera il campo'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  Check 'riaperta' (IsOpen)
  Check 'campo vuoto' ((Js "document.querySelector('.launcher-input').value") -eq '')

  Write-Host 'F. Invio avvia l''app e chiude'
  TypeText 'calc'; Start-Sleep -Milliseconds 700
  Press 0x0D; Start-Sleep -Seconds 3
  Check 'Launcher chiuso' (-not (IsOpen))
  $calc = @(Get-Process | Where-Object { $_.ProcessName -like 'Calculator*' }).Count
  Check 'Calcolatrice avviata' ($calc -gt 0)
  $usage = Get-Content (Join-Path $env:LOCALAPPDATA 'EosOS\launcher.json') -Raw
  Check 'uso registrato' ($usage -match 'WindowsCalculator')

  Write-Host 'G. perdita di focus nasconde'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  Check 'aperta' (IsOpen)
  $cw = [LT]::Find('Calcolatrice')
  if ($cw -ne [IntPtr]::Zero) { [void][LT]::SetForegroundWindow($cw); Start-Sleep -Milliseconds 800 }
  Check 'nascosta dopo aver perso il focus' (-not (IsOpen))

  Write-Host 'H. la scorciatoia chiude'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  Check 'aperta' (IsOpen)
  [LT]::AltSpace(); Start-Sleep -Milliseconds 700
  Check 'chiusa dalla stessa scorciatoia' (-not (IsOpen))

  Write-Host 'I. calcolatrice nel campo'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  Check 'aperta' (IsOpen)
  # '*' con il tasto del tastierino numerico (VK_MULTIPLY = 0x6A): nessuna dipendenza dal layout.
  TypeText '12'; Press 0x6A; TypeText '7'; Start-Sleep -Milliseconds 800
  Check 'campo = 12*7' ((Js "document.querySelector('.launcher-input').value") -eq '12*7') ((Js "document.querySelector('.launcher-input').value"))
  Check 'primo risultato e il calcolo' ((Js "document.querySelector('.launcher-item').dataset.kind") -eq 'calc')
  Check 'risultato 84' ((Js "document.querySelector('.launcher-name').textContent") -eq '84')
  Press 0x0D; Start-Sleep -Milliseconds 800
  Check 'Launcher chiuso dopo la copia' (-not (IsOpen))
  Check 'risultato negli appunti' ((Get-Clipboard -Raw).Trim() -eq '84')

  Write-Host 'J. schede con Ctrl+1 / Ctrl+2'
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  TypeText 'impost'; Start-Sleep -Milliseconds 700
  $kindsAll = Js "Array.from(document.querySelectorAll('.launcher-item')).map(function(e){return e.dataset.kind}).join(',')"
  Check 'senza scheda: azione Impostazioni presente (o app omonima)' ($kindsAll -match 'action|app') $kindsAll
  PressCtrl 0x32; Start-Sleep -Milliseconds 700
  Check 'chip Azioni visibile' ((Js "!!document.querySelector('.launcher-chip') && document.querySelector('.launcher-chip').textContent") -eq 'Azioni')
  $kinds = Js "Array.from(document.querySelectorAll('.launcher-item')).map(function(e){return e.dataset.kind}).join(',')"
  Check 'solo azioni' ($kinds -eq 'action') $kinds
  PressCtrl 0x32; Start-Sleep -Milliseconds 600
  Check 'secondo Ctrl+2 toglie la scheda' ((Js "!document.querySelector('.launcher-chip')") -eq $true)
  PressCtrl 0x31; Start-Sleep -Milliseconds 600
  $kinds = Js "Array.from(document.querySelectorAll('.launcher-item')).map(function(e){return e.dataset.kind}).join(',')"
  Check 'Ctrl+1: solo app' (($kinds -eq 'app') -or ($kinds -eq '')) "kinds=$kinds"

  Write-Host 'K. conferma di Riavvia (build di prova: non esegue)'
  PressCtrl 0x31; Start-Sleep -Milliseconds 400   # toglie la scheda App
  foreach ($i in 1..6) { Press 0x08 }
  TypeText 'riavvia'; Start-Sleep -Milliseconds 800
  Check 'primo risultato e Riavvia' ((Js "document.querySelector('.launcher-item').id") -eq 'launcher-item-0' -and (Js "document.querySelector('.launcher-item').dataset.kind") -eq 'action')
  Press 0x0D; Start-Sleep -Milliseconds 600
  Check 'primo Invio: chiede conferma' ((Js "document.querySelector('.launcher-item').dataset.confirming") -eq 'true')
  Check 'ancora aperto e nulla eseguito' ((IsOpen) -and -not (Test-Path $actionLog))
  TypeText 'a'; Start-Sleep -Milliseconds 700   # digitare annulla la conferma
  Check 'digitare annulla la conferma' ((Js "document.querySelector('.launcher-item') ? document.querySelector('.launcher-item').dataset.confirming : 'x'") -ne 'true')
  foreach ($i in 1..1) { Press 0x08 }; Start-Sleep -Milliseconds 700
  Press 0x0D; Start-Sleep -Milliseconds 500
  Check 'dopo l''annullamento serve di nuovo la conferma e nulla e eseguito' ((Js "document.querySelector('.launcher-item').dataset.confirming") -eq 'true' -and -not (Test-Path $actionLog))
  Press 0x0D; Start-Sleep -Milliseconds 800
  Check 'secondo Invio: Launcher chiuso' (-not (IsOpen))
  $log = if (Test-Path $actionLog) { Get-Content $actionLog -Raw } else { '' }
  Check 'azione registrata (non eseguita) una sola volta' (([regex]::Matches($log, 'action:restart')).Count -eq 1) $log.Trim()

  Write-Host 'L. Impostazioni (azione normale, senza conferma)'
  $settingsBefore = [LT]::Find('Impostazioni')
  if ($settingsBefore -ne [IntPtr]::Zero) { Write-Host '  (Impostazioni gia aperta: provo solo che il Launcher si chiuda)' }
  [LT]::AltSpace(); Start-Sleep -Milliseconds 900
  PressCtrl 0x32; Start-Sleep -Milliseconds 300
  TypeText 'impost'; Start-Sleep -Milliseconds 700
  Press 0x0D; Start-Sleep -Seconds 3
  Check 'Launcher chiuso' (-not (IsOpen))
  if ($settingsBefore -eq [IntPtr]::Zero) {
    $sw = [LT]::Find('Impostazioni')
    Check 'finestra Impostazioni aperta' ($sw -ne [IntPtr]::Zero)
    if ($sw -ne [IntPtr]::Zero) { [void][LT]::PostClose($sw) }
  }
  Check 'nessuna nuova azione di alimentazione' ((Get-Content $actionLog -Raw) -notmatch 'settings|lock|sleep|shutdown')
}
finally {
  Cleanup
  if ($savedClip) { Set-Clipboard -Value $savedClip }
  Remove-Item $actionLog -ErrorAction SilentlyContinue
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
