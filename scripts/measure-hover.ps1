# Fluidita' del Dock al passaggio del mouse: la pagina del Dock simula movimenti del cursore lungo la capsula e misura quanto
# dura ogni fotogramma (requestAnimationFrame). Si confronta con la sfocatura accesa e spenta (impostazione, ripristinata).
# Non si muove il mouse vero: gli eventi sono simulati nella pagina. dock.json salvato e ripristinato per intero.
# Uso: powershell -File scripts\measure-hover.ps1 [-Exe percorso\eosos.exe]
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if (-not $Exe) { $Exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
$dj = Join-Path $env:LOCALAPPDATA 'EosOS\dock.json'
$bk = if (Test-Path $dj) { [System.IO.File]::ReadAllBytes($dj) } else { $null }
$script = @'
(async function(){
  var zone=document.querySelector('.dock-zone'); var pill=document.querySelector('.dock:not(.dock-ghost)');
  var r=pill.getBoundingClientRect(); var frames=[]; var last=performance.now(); var run=true;
  function loop(t){ frames.push(t-last); last=t; if(run) requestAnimationFrame(loop); }
  requestAnimationFrame(loop);
  var t0=performance.now(); var i=0;
  while(performance.now()-t0<3000){
    var x=r.left+10+((Math.sin(i/12)+1)/2)*(r.width-20); var y=r.top+r.height/2;
    zone.dispatchEvent(new MouseEvent('mousemove',{bubbles:true,clientX:x,clientY:y}));
    i++; await new Promise(function(res){setTimeout(res,8)});
  }
  run=false; zone.dispatchEvent(new MouseEvent('mouseleave',{bubbles:true}));
  frames=frames.slice(5); frames.sort(function(a,b){return a-b});
  var avg=frames.reduce(function(a,b){return a+b},0)/frames.length;
  return JSON.stringify({n:frames.length,avg:Math.round(avg*10)/10,p50:Math.round(frames[Math.floor(frames.length*0.5)]*10)/10,p95:Math.round(frames[Math.floor(frames.length*0.95)]*10)/10,max:Math.round(frames[frames.length-1]*10)/10,over25:frames.filter(function(f){return f>25}).length});
})()
'@
try {
  Cleanup-Eos; Start-Sleep 1
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $Exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 14
  foreach ($blur in $true, $false, $true) {
    [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'dock.blur',value:$($blur.ToString().ToLower())})")
    Start-Sleep 2
    $r = Js 'dock' ($script -replace "`r?`n", ' ')
    $o = $r | ConvertFrom-Json
    Write-Host ("sfocatura {0,-6}: fotogrammi {1}, media {2} ms, mediana {3}, 95% {4}, massimo {5}, oltre 25 ms: {6}" -f $(if ($blur) { 'ACCESA' } else { 'spenta' }), $o.n, $o.avg, $o.p50, $o.p95, $o.max, $o.over25)
  }
}
finally {
  Cleanup-Eos
  if ($null -ne $bk) { [System.IO.File]::WriteAllBytes($dj, $bk) }
}

# La build di produzione nasconde la barra di Windows e fermarla a forza (tutti gli eosos.exe, watchdog compreso) non la ripristina: si rimette qui.
& (Join-Path $PSScriptRoot "restore-taskbar.ps1")
