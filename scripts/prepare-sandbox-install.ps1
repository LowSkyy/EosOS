# Prepara la prova dell'INSTALLATORE in Windows Sandbox: copia il setup NSIS (da `npx tauri build`) e lo script in
# sandbox\in-install e genera sandbox\EosOS-install.wsb. Poi basta aprire il .wsb con doppio clic (apre una finestra
# di Windows Sandbox: la prova gira da sola; l'installazione resta dentro la Sandbox e sparisce alla chiusura).
# I risultati compaiono in sandbox\out\risultati-install.txt.
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$setup = Get-ChildItem (Join-Path $root 'src-tauri\target\release\bundle\nsis\*setup*.exe') | Sort-Object LastWriteTime -Descending | Select-Object -First 1
if (-not $setup) { throw "Manca il setup NSIS: esegui prima 'npx tauri build'." }
$in = Join-Path $root 'sandbox\in-install'; $out = Join-Path $root 'sandbox\out'
Remove-Item $in -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$in\scripts", $out | Out-Null
Remove-Item "$out\risultati-install.txt" -Force -ErrorAction SilentlyContinue
Copy-Item $setup.FullName $in
Copy-Item (Join-Path $PSScriptRoot 'sandbox-install.ps1') "$in\scripts"
$wsb = @"
<Configuration>
  <VGpu>Enable</VGpu>
  <Networking>Disable</Networking>
  <MappedFolders>
    <MappedFolder><HostFolder>$in</HostFolder><SandboxFolder>C:\EosOS</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$out</HostFolder><SandboxFolder>C:\EosOS-out</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell -ExecutionPolicy Bypass -File C:\EosOS\scripts\sandbox-install.ps1</Command></LogonCommand>
</Configuration>
"@
Set-Content -Path (Join-Path $root 'sandbox\EosOS-install.wsb') -Value $wsb -Encoding utf8
Write-Host "Installatore: $($setup.Name) ($([math]::Round($setup.Length/1MB,1)) MB)"
Write-Host "Pronto: apri $(Join-Path $root 'sandbox\EosOS-install.wsb') (doppio clic)."
Write-Host "Risultati in: $out\risultati-install.txt"
