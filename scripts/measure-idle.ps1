# Misura a riposo: processi di EosOS (e solo i suoi WebView2, trovati dall'albero dei processi), memoria, maniglie e CPU in 30 s.
# Non tocca nessun altro processo. Uso: powershell -File scripts\measure-idle.ps1 [-Exe percorso] [-Seconds 30]
param([string]$Exe, [int]$Seconds = 30)
if (-not $Exe) { $Exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
Start-Sleep 2
$p = Start-Process $Exe -PassThru
Start-Sleep 25
function Tree($root) { $all = Get-CimInstance Win32_Process; $ids = [System.Collections.Generic.HashSet[int]]::new(); [void]$ids.Add($root); do { $n = $ids.Count; $all | Where-Object { $ids.Contains([int]$_.ParentProcessId) } | ForEach-Object { [void]$ids.Add([int]$_.ProcessId) } } while ($ids.Count -ne $n); $ids }
$cmd = @{}; Get-CimInstance Win32_Process | ForEach-Object { $cmd[[int]$_.ProcessId] = $_.CommandLine }
function Snap { $h = @{}; foreach ($i in (Tree $p.Id)) { $x = Get-Process -Id $i -ErrorAction SilentlyContinue; if ($x) { $h[$i] = [pscustomobject]@{ cpu = $x.CPU; h = $x.HandleCount; mb = ($x.WorkingSet64 / 1MB); name = $x.ProcessName } } }; $h }
$a = Snap; Start-Sleep $Seconds; $b = Snap
$rows = foreach ($k in $b.Keys) { if ($a.ContainsKey($k)) { $t = (($cmd[$k] -split ' ') | Where-Object { $_ -like '--type=*' }) -join ''; $role = if ($cmd[$k] -match 'keyhook') { 'keyhook' } else { $t }; [pscustomobject]@{ pid = $k; name = $b[$k].name; role = $role; dCPU = [math]::Round($b[$k].cpu - $a[$k].cpu, 2); h0 = $a[$k].h; h1 = $b[$k].h; MB = [math]::Round($b[$k].mb) } } }
$rows | Sort-Object dCPU -Descending | Format-Table -AutoSize
"Totale: processi=$($rows.Count) MB=$([math]::Round(($rows | Measure-Object MB -Sum).Sum)) CPU in ${Seconds}s=$([math]::Round(($rows | Measure-Object dCPU -Sum).Sum, 2)) s  maniglie=$(($rows | Measure-Object h1 -Sum).Sum)"
Stop-Process -Id $p.Id -Force -ErrorAction SilentlyContinue
Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue }
