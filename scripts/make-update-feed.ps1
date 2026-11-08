# Prepara i file da pubblicare per un aggiornamento: legge l'installatore NSIS firmato prodotto da `npx tauri build`
# (con la chiave privata) e scrive `latest.json`, che EosOS scarica per sapere se c'e una versione nuova.
# NON pubblica niente: i file finiscono in dist-update\ e li carichi tu (per esempio come allegati di una release su GitHub:
# EosOS_<versione>_x64-setup.exe, il suo .sig e latest.json).
#
# Uso: powershell -File scripts\make-update-feed.ps1 -BaseUrl https://github.com/<utente>/<repo>/releases/download/v<versione> [-Notes "testo"]
param(
  [Parameter(Mandatory = $true)][string]$BaseUrl,
  [string]$Notes = ''
)
$ErrorActionPreference = 'Stop'
$root = Resolve-Path (Join-Path $PSScriptRoot '..')
if ($BaseUrl -notmatch '^https://') { throw 'BaseUrl deve cominciare con https:// (EosOS rifiuta gli aggiornamenti non in https).' }
$conf = Get-Content (Join-Path $root 'src-tauri\tauri.conf.json') -Raw | ConvertFrom-Json
$version = $conf.version
$setup = Join-Path $root "src-tauri\target\release\bundle\nsis\EosOS_${version}_x64-setup.exe"
$sigFile = "$setup.sig"
if (-not (Test-Path $setup) -or -not (Test-Path $sigFile)) { throw "Mancano $setup e/o il suo .sig: esegui 'npx tauri build' con TAURI_SIGNING_PRIVATE_KEY (il contenuto del file della chiave) impostata." }
$sig = (Get-Content $sigFile -Raw).Trim()
$name = Split-Path $setup -Leaf
$url = ($BaseUrl.TrimEnd('/')) + '/' + $name
$entry = @{ signature = $sig; url = $url }
$feed = [ordered]@{
  version = $version
  notes = $Notes
  pub_date = (Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ')
  platforms = [ordered]@{ 'windows-x86_64' = $entry; 'windows-x86_64-nsis' = $entry }
}
$out = Join-Path $root 'dist-update'
New-Item -ItemType Directory -Force $out | Out-Null
Copy-Item $setup $out -Force
Copy-Item $sigFile $out -Force
# Senza BOM: serde_json (EosOS) rifiuta un feed che comincia con il BOM.
[System.IO.File]::WriteAllText((Join-Path $out 'latest.json'), ($feed | ConvertTo-Json -Depth 5), (New-Object System.Text.UTF8Encoding($false)))
Write-Host "Versione $version pronta in $out :"
Get-ChildItem $out | ForEach-Object { Write-Host ("  " + $_.Name + "  (" + [math]::Round($_.Length / 1KB) + " KB)") }
Write-Host "Carica questi tre file nella stessa posizione di $BaseUrl (latest.json va dove lo cerca EosOS, vedi docs/AGGIORNAMENTI.md)."
