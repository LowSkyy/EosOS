# Prepara la prova in Windows Sandbox: copia eosos.exe (release) e gli script in sandbox\in
# e genera sandbox\EosOS.wsb. Poi basta aprire il .wsb con doppio clic.
# Requisiti: Windows Sandbox abilitato (Funzionalita' di Windows > Windows Sandbox, riavvio).
# I risultati compaiono in sandbox\out\ (risultati.txt) quando la prova automatica e' finita.
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
$exe = Join-Path $root 'src-tauri\target\release\eosos.exe'
if (-not (Test-Path $exe)) { throw "Manca ${exe}: esegui prima 'npm run tauri build'." }
$in = Join-Path $root 'sandbox\in'; $out = Join-Path $root 'sandbox\out'
Remove-Item $in -Recurse -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force "$in\scripts", $out | Out-Null
Remove-Item "$out\*" -Force -ErrorAction SilentlyContinue
Copy-Item $exe $in
foreach ($s in 'test-taskbar.ps1','test-workarea.ps1','sandbox-run.ps1') { Copy-Item (Join-Path $PSScriptRoot $s) "$in\scripts" }
$wsb = @"
<Configuration>
  <VGpu>Enable</VGpu>
  <Networking>Disable</Networking>
  <MappedFolders>
    <MappedFolder><HostFolder>$in</HostFolder><SandboxFolder>C:\EosOS</SandboxFolder><ReadOnly>true</ReadOnly></MappedFolder>
    <MappedFolder><HostFolder>$out</HostFolder><SandboxFolder>C:\EosOS-out</SandboxFolder><ReadOnly>false</ReadOnly></MappedFolder>
  </MappedFolders>
  <LogonCommand><Command>powershell -ExecutionPolicy Bypass -File C:\EosOS\scripts\sandbox-run.ps1</Command></LogonCommand>
</Configuration>
"@
Set-Content -Path (Join-Path $root 'sandbox\EosOS.wsb') -Value $wsb -Encoding utf8
Write-Host "Pronto: apri $(Join-Path $root 'sandbox\EosOS.wsb') (doppio clic)."
Write-Host "Risultati in: $out\risultati.txt"
