# Rete di sicurezza di test-dock-scales.ps1 e test-dock-mixed-dpi.ps1: dopo $After secondi rimette
# la scala del monitor $Gdi (di default il primario) a $Orig se non e' gia' quella. Cosi' un crollo dello script (o della sessione) non
# lascia lo schermo a una scala diversa da quella dell'utente.
param([int]$Orig, [int]$After = 600, [string]$Gdi = '')
. "$PSScriptRoot\lib-dpi.ps1"
Start-Sleep -Seconds $After
if (-not $Gdi) { $Gdi = Get-PrimaryGdi }
if ((Get-MonitorScale $Gdi) -ne $Orig) { try { [void](Set-MonitorScale $Gdi $Orig) } catch {} }
