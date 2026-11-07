# Gira DENTRO Windows Sandbox (lanciato dal .wsb). Esegue le prove sulla build release
# con la taskbar nascosta, scrive i risultati in C:\EosOS-out\risultati.txt e infine
# lascia EosOS attivo con la taskbar nascosta per la prova manuale.
$out = 'C:\EosOS-out\risultati.txt'
$exe = 'C:\EosOS\eosos.exe'
Start-Sleep 20   # lascia finire l'avvio del desktop della Sandbox
function Log($t) { Add-Content -Path $out -Value $t -Encoding utf8 }
Set-Content -Path $out -Value "Prova in Windows Sandbox - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -Encoding utf8
Log "OS: $((Get-CimInstance Win32_OperatingSystem).Caption) $((Get-CimInstance Win32_OperatingSystem).Version)"
Log "Utente: $env:USERNAME"
try {
    # La cartella C:\EosOS e' di sola lettura: il marcatore e il log vanno in LOCALAPPDATA, che e' scrivibile.
    Log "`n=== test-workarea (release) ==="
    Log ((& powershell -NoProfile -ExecutionPolicy Bypass -File C:\EosOS\scripts\test-workarea.ps1 -Exe $exe 2>&1 | Out-String))
    Log "`n=== test-taskbar (release, senza panic) ==="
    Log ((& powershell -NoProfile -ExecutionPolicy Bypass -File C:\EosOS\scripts\test-taskbar.ps1 -Exe $exe -SkipPanic 2>&1 | Out-String))
    Log "`nLog watchdog:"
    $wl = Join-Path $env:LOCALAPPDATA 'EosOS\logs\watchdog.log'
    if (Test-Path $wl) { Log ((Get-Content $wl -Tail 20 | Out-String)) } else { Log '(nessun log)' }
} catch { Log "ERRORE: $_" }
Log "`nPROVA AUTOMATICA FINITA. Ora EosOS e' avviato con la taskbar nascosta per la prova manuale."
$env:EOSOS_HIDE_TASKBAR = '1'
Start-Process $exe
