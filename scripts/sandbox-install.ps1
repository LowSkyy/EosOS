# Gira DENTRO Windows Sandbox (lanciato da sandbox\EosOS-install.wsb): prova l'INSTALLATORE vero, non l'exe sciolto.
# Installa in silenzio con NSIS, avvia EosOS, controlla che resti vivo e che le finestre ci siano, poi lo chiude,
# lo disinstalla e controlla che non resti niente (taskbar e area di lavoro compresi). Scrive tutto in
# C:\EosOS-out\risultati-install.txt. Roba solo della Sandbox: l'installazione NON tocca il PC vero.
$out = 'C:\EosOS-out\risultati-install.txt'
function Log($t) { Add-Content -Path $out -Value $t -Encoding utf8 }
function Check($l, $c, $d = '') { Log ($(if ($c) { '[OK]      ' } else { '[FALLITO] ' }) + "$l $d") }
Start-Sleep 20
Set-Content -Path $out -Value "Prova dell'installatore in Windows Sandbox - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -Encoding utf8
Log "OS: $((Get-CimInstance Win32_OperatingSystem).Caption) $((Get-CimInstance Win32_OperatingSystem).Version); utente $env:USERNAME"
try {
  $setup = Get-ChildItem 'C:\EosOS\*setup*.exe' | Select-Object -First 1
  Check 'installatore presente' ($null -ne $setup) $setup.Name
  # L'area di lavoro vera (SPI_GETWORKAREA), come in test-workarea.ps1: Screen.WorkingArea di .NET puo' restare in cache.
  Add-Type @'
using System; using System.Runtime.InteropServices;
public static class WA {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] static extern bool SystemParametersInfo(uint a, uint b, ref RECT r, uint c);
  public static string Get() { RECT r = new RECT(); SystemParametersInfo(0x30, 0, ref r, 0); return r.T + ".." + r.B; }
}
'@
  $wa0 = [WA]::Get()
  $p = Start-Process $setup.FullName -ArgumentList '/S' -PassThru -Wait
  Check 'installazione silenziosa riuscita' ($p.ExitCode -eq 0) "codice $($p.ExitCode)"
  $dir = Join-Path $env:LOCALAPPDATA 'EosOS'
  $exe = Get-ChildItem -Path $env:LOCALAPPDATA, $env:ProgramFiles -Filter 'eosos.exe' -Recurse -ErrorAction SilentlyContinue | Where-Object { $_.FullName -notmatch 'sandbox|C:\\EosOS\\' } | Select-Object -First 1
  Check 'eosos.exe installato' ($null -ne $exe) $exe.FullName
  $uninst = Get-ChildItem -Path (Split-Path $exe.FullName) -Filter 'uninstall*.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
  Check 'disinstallatore presente' ($null -ne $uninst)
  $lnk = Get-ChildItem "$env:APPDATA\Microsoft\Windows\Start Menu\Programs" -Recurse -Filter 'EosOS*.lnk' -ErrorAction SilentlyContinue | Select-Object -First 1
  Check 'collegamento nel menu Start' ($null -ne $lnk)

  $env:EOSOS_HIDE_TASKBAR = '0'
  Start-Process $exe.FullName
  Start-Sleep 25
  $procs = @(Get-Process eosos -ErrorAction SilentlyContinue)
  Check 'EosOS e in esecuzione dopo 25 s' ($procs.Count -ge 1) "processi: $($procs.Count)"
  Check 'le finestre WebView2 sono partite' (@(Get-Process msedgewebview2 -ErrorAction SilentlyContinue).Count -ge 3)
  Check 'nessun avviso di errore a schermo (nessuna finestra di errore)' (@(Get-Process | Where-Object { $_.MainWindowTitle -match 'errore|error|WebView2 Runtime' }).Count -eq 0)
  $wa1 = [WA]::Get()
  Check 'la barra riserva spazio (l area di lavoro cambia)' ($wa1 -ne $wa0) "area di lavoro $wa0 -> $wa1"
  Start-Sleep 30
  Check 'ancora vivo dopo 55 s' (@(Get-Process eosos -ErrorAction SilentlyContinue).Count -ge 1)

  Get-Process eosos -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep 5
  $wa2 = [WA]::Get()
  Check 'dopo la chiusura l area di lavoro torna intera' ($wa2 -eq $wa0) "prima $wa0, dopo $wa2"
  if ($uninst) {
    $u = Start-Process $uninst.FullName -ArgumentList '/S' -PassThru -Wait
    Check 'disinstallazione silenziosa riuscita' ($u.ExitCode -eq 0) "codice $($u.ExitCode)"
    Start-Sleep 5
    Check 'eosos.exe non c e piu' (-not (Test-Path $exe.FullName))
  }
} catch { Log "ERRORE: $_" }
Log "`nPROVA FINITA."
