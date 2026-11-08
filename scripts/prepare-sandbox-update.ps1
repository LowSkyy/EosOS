# Prepara la prova dell'AGGIORNAMENTO in Windows Sandbox. Prima servono (vedi docs/AGGIORNAMENTI.md):
#   sandbox\in-update\EosOS_A_0.1.0-setup.exe        la 0.1.0 costruita con il feed locale (TAURI_CONFIG, solo per la prova)
#   sandbox\in-update\feed\EosOS_0.1.1_x64-setup.exe e il suo .sig    la 0.1.1 firmata con la chiave vera
# Genera sandbox\EosOS-update.wsb: aprilo con un doppio clic (si apre una finestra di Windows Sandbox, la prova gira da sola
# e scrive in sandbox\out\risultati-update.txt; l'installazione resta dentro la Sandbox).
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$in = Join-Path $root 'sandbox\in-update'; $out = Join-Path $root 'sandbox\out'
foreach ($f in 'EosOS_A_0.1.0-setup.exe', 'feed\EosOS_0.1.1_x64-setup.exe', 'feed\EosOS_0.1.1_x64-setup.exe.sig') { if (-not (Test-Path (Join-Path $in $f))) { throw "Manca $in\$f" } }
New-Item -ItemType Directory -Force "$in\scripts", $out | Out-Null
Remove-Item "$out\risultati-update.txt" -Force -ErrorAction SilentlyContinue
Copy-Item (Join-Path $PSScriptRoot 'sandbox-update.ps1') "$in\scripts" -Force
$wsb = @"
<Configuration>
  <VGpu>Enable</VGpu>
  <Networking>Disable</Networking>
  <MappedFolders>
    <MappedFolder><HostFolder>$in</HostFolder><SandboxFolder>C:\EosOS</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$out</HostFolder><SandboxFolder>C:\EosOS-out</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell -ExecutionPolicy Bypass -File C:\EosOS\scripts\sandbox-update.ps1</Command></LogonCommand>
</Configuration>
"@
Set-Content -Path (Join-Path $root 'sandbox\EosOS-update.wsb') -Value $wsb -Encoding utf8
Write-Host "Pronto: apri $(Join-Path $root 'sandbox\EosOS-update.wsb') (doppio clic). Risultati in: $out\risultati-update.txt"
