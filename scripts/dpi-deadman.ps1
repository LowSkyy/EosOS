# Rete di sicurezza di test-dock-scales.ps1: dopo $After secondi rimette la scala del monitor
# primario a $Orig se non e' gia' quella. Cosi' un crollo dello script (o della sessione) non
# lascia lo schermo a una scala diversa da quella dell'utente.
param([int]$Orig, [int]$After = 600)
. "$PSScriptRoot\lib-dpi.ps1"
Start-Sleep -Seconds $After
if ((Get-PrimaryScale) -ne $Orig) { try { [void](Set-PrimaryScale $Orig) } catch {} }
