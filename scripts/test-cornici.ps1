# Cornici: nessuna finestra di EosOS deve mai avere, nemmeno per un fotogramma, la barra del titolo di Windows (la barra azzurra «EosOS Dock» sopra il Dock).
# Si campionano gli stili di TUTTE le finestre VISIBILI del processo (una finestra nascosta con la cornice non si vede) ogni ~2 ms per qualche secondo mentre si aprono e si chiudono molte volte le Impostazioni, il
# pannello sotto l'orologio e il Launcher; si conta quante volte compare WS_CAPTION / WS_SYSMENU. Deve essere zero. Non si clicca niente di vero.
#
# Uso: powershell -File scripts\test-cornici.ps1 [-Exe percorso\eosos.exe] [-Giri 25]   (serve una build con test-hooks)
param([string]$Exe, [int]$Giri = 25)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic; using System.Threading;
public static class CO {
 public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
 [DllImport("user32.dll")] static extern long GetWindowLongPtr(IntPtr h, int i);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 static volatile bool stop; static int samples; static int framed; static string first = ""; static Thread t;
 public static void Start(uint pid){ stop=false; samples=0; framed=0; first=""; t=new Thread(()=>{ while(!stop){ EnumWindows((h,x)=>{ uint p; GetWindowThreadProcessId(h,out p); if(p==pid){ long st=GetWindowLongPtr(h,-16); if((st & 0xCB0000)!=0 && (st & 0x10000000)!=0){ framed++; if(first==""){ var sb=new StringBuilder(100); GetWindowText(h,sb,100); first=sb.ToString()+" stile 0x"+st.ToString("X"); } } } return true; },IntPtr.Zero); samples++; Thread.Sleep(2); } }); t.IsBackground=true; t.Start(); }
 [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out int v, int s);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 // Le finestre VISIBILI e con un titolo (non la finestrella di servizio «Tao Thread Event Target», 16x16 px) che hanno il rendering non client di DWM spento (con quello spento Windows disegnava la barra azzurra «EosOS Dock»).
 public static string NcSpento(uint pid){ var o=new StringBuilder(); EnumWindows((h,x)=>{ uint p; GetWindowThreadProcessId(h,out p); if(p==pid && IsWindowVisible(h)){ int v=1; DwmGetWindowAttribute(h,1,out v,4); if(v==0){ var sb=new StringBuilder(100); GetWindowText(h,sb,100); if(sb.Length>0) o.Append(sb.ToString()+"; "); } } return true; },IntPtr.Zero); return o.ToString(); }
 public static string Stop(){ stop=true; t.Join(); return samples+"|"+framed+"|"+first; }
}
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { & node $cdp eval $expr 2>$null } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
try {
  Cleanup-Eos; Start-Sleep 1; Remove-Item $mc -ErrorAction SilentlyContinue
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  $proc = Start-Process $exe -PassThru
  Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  $pid0 = [uint32]$proc.Id
  # Si campiona fin dall'avvio: le barre in alto e i Dock si creano li' e non devono avere la cornice nemmeno per un istante.
  [CO]::Start($pid0)
  Start-Sleep 14
  $a = ([CO]::Stop()) -split '\|'
  Write-Host ("  avvio: campioni " + $a[0] + ", con cornice " + $a[1] + " " + $a[2])
  Check 'dall avvio: nessuna finestra (barre in alto, Dock, pannelli...) ha avuto la cornice' ([int]$a[1] -eq 0) ("prima: " + $a[2])
  $nc = [CO]::NcSpento($pid0)
  Check 'tutte le finestre visibili di EosOS hanno il rendering non client DWM acceso (niente barra azzurra sopra il Dock)' ([string]::IsNullOrEmpty($nc)) ("spento su: " + $nc)
  [CO]::Start($pid0)
  Write-Host "campionamento degli stili avviato (processo $pid0): $Giri aperture e chiusure"
  for ($i = 0; $i -lt $Giri; $i++) {
    Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_open',{section:null})" | Out-Null; Start-Sleep -Milliseconds 220
    Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_close')" | Out-Null; Start-Sleep -Milliseconds 120
    Js 'bar' "document.querySelector('.clock').click()" | Out-Null; Start-Sleep -Milliseconds 220
    Js 'launcher' "window.__TAURI_INTERNALS__.invoke('panel_hide')" | Out-Null; Start-Sleep -Milliseconds 120
    Js 'launcher' "window.__TAURI_INTERNALS__.invoke('launcher_action',{id:'noop'})" | Out-Null
    Js 'dock' "document.querySelector('.dock-launchpad').click()" | Out-Null; Start-Sleep -Milliseconds 300
    Js 'launcher' "window.__TAURI_INTERNALS__.invoke('launcher_hide')" | Out-Null; Start-Sleep -Milliseconds 120
  }
  $r = ([CO]::Stop()) -split '\|'
  Write-Host ("  campioni: " + $r[0] + ", con cornice: " + $r[1] + " " + $r[2])
  Check 'quasi mille campioni di tutte le finestre' ([int]$r[0] -ge 500) ("campioni " + $r[0])
  Check 'nessuna finestra di EosOS ha avuto la cornice, nemmeno per un istante' ([int]$r[1] -eq 0) ("prima: " + $r[2])
}
finally {
  Cleanup-Eos
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
