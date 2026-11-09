# Cattura dello schermo: schermo intero, finestra in primo piano e selezione, su una finestra di prova MIA di colore noto (magenta). Le immagini
# vanno in una cartella temporanea (EOSOS_CAPTURE_DIR), non in Immagini\Screenshots; di base senza toccare appunti e suono (EOSOS_TEST_QUIET).
# Si controlla: file PNG con la misura giusta, il contenuto (il magenta della finestra di prova), che dopo lo scatto le finestre di EosOS tornino escluse dalla
# cattura (come prima), il velo della selezione (trascinamento, annullo con Esc, clic senza trascinare), il controllo accessi e, a parte, gli appunti
# (il testo degli appunti e' salvato e rimesso; se negli appunti c'e' altro non si prova). Impostazioni: spente di base, 3 combinazioni scegliendo un modificatore.
#
# Uso: powershell -File scripts\test-capture.ps1 [-Exe percorso\eosos.exe]   (serve una build con test-hooks)
param([string]$Exe)
$ErrorActionPreference = 'Stop'
if ($Exe) { $exe = (Resolve-Path $Exe).Path } else { $exe = (Join-Path $PSScriptRoot '..\src-tauri\target\fast\release\eosos.exe' | Resolve-Path).Path }
$cdp = Join-Path $PSScriptRoot 'cdp-dock.mjs'
Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);' -Name Dpi -Namespace CapTest
[void][CapTest.Dpi]::SetProcessDpiAwarenessContext([IntPtr](-4))
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class CT { public delegate bool EP(IntPtr h, IntPtr l);
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
 [DllImport("user32.dll")] static extern bool GetWindowDisplayAffinity(IntPtr h, out uint a);
 [DllImport("user32.dll")] static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("dwmapi.dll")] static extern int DwmGetWindowAttribute(IntPtr h, int a, out R r, int n);
 public static bool Visible(string part){ bool f=false; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString().Contains(part)){f=true;return false;} return true;},IntPtr.Zero); return f; }
 public static int[] Rect(string title){ int[] o=null; EnumWindows((h,l)=>{var sb=new StringBuilder(256);GetWindowText(h,sb,256); if(IsWindowVisible(h)&&sb.ToString()==title){R r;GetWindowRect(h,out r);o=new int[]{r.L,r.T,r.Rt,r.B};return false;} return true;},IntPtr.Zero); return o; }
 public static string Affinities(uint pid){ var l=new List<string>(); EnumWindows((h,x)=>{ uint p; GetWindowThreadProcessId(h,out p); if(p==pid){ uint a; if(GetWindowDisplayAffinity(h,out a)) l.Add(h.ToInt64()+":"+a);} return true;},IntPtr.Zero); l.Sort(); return string.Join(",",l); }
 public static int[] Frame(long h){ R r; if (DwmGetWindowAttribute(new IntPtr(h), 9, out r, 16) != 0) GetWindowRect(new IntPtr(h), out r); return new int[]{ r.L, r.T, r.Rt, r.B }; }
 [DllImport("user32.dll")] static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int hh, uint f);
 public static void MoveTo(long h, int x, int y){ SetWindowPos(new IntPtr(h), IntPtr.Zero, x, y, 0, 0, 0x15); }
 public static void ToTop(long h){ SetWindowPos(new IntPtr(h), IntPtr.Zero, 0, 0, 0, 0, 0x13); }
 [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
 [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(P p);
 [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
 public static bool IsTopAt(long h, int x, int y){ P p; p.X=x; p.Y=y; IntPtr w = WindowFromPoint(p); return w != IntPtr.Zero && GetAncestor(w, 2).ToInt64() == h; }
 public static long Foreground(){ return GetForegroundWindow().ToInt64(); }
 public static void Front(long h){ SetForegroundWindow(new IntPtr(h)); } }
'@
$script:fail = 0
function Check($l, $c, $d = '') { if ($c) { Write-Host "  OK   $l $d" } else { Write-Host "  FAIL $l $d" -ForegroundColor Red; $script:fail++ } }
function Cleanup-Eos { Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | ForEach-Object { Stop-Process -Id $_.ProcessId -Force -ErrorAction SilentlyContinue } }
function Js([string]$view, [string]$expr) { $env:CDP_VIEW = $view; $env:CDP_INDEX = '0'; $old = $ErrorActionPreference; $ErrorActionPreference = 'Continue'; try { $o = & node $cdp eval $expr 2>$null; if ($o) { try { $o | ConvertFrom-Json } catch { $null } } } finally { $ErrorActionPreference = $old; Remove-Item env:CDP_INDEX -ErrorAction SilentlyContinue } }
function Wait-Until([scriptblock]$cond, [double]$seconds = 8) { $t0 = Get-Date; while (((Get-Date) - $t0).TotalSeconds -lt $seconds) { if (& $cond) { return $true }; Start-Sleep -Milliseconds 200 }; $false }
function Set-S([string]$key, [string]$jsValue) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_set',{key:'$key',value:$jsValue})" }
function Fire([string]$kind, [long]$hwnd = 0) { if ($hwnd -ne 0) { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_fire',{kind:'$kind',hwnd:$hwnd})" } else { Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_fire',{kind:'$kind'})" } }
function Png-Size([string]$path) { $b = [System.IO.File]::ReadAllBytes($path); if ($b.Length -lt 24 -or $b[0] -ne 0x89 -or $b[1] -ne 0x50) { return $null }; $w = ([int]$b[16] -shl 24) + ([int]$b[17] -shl 16) + ([int]$b[18] -shl 8) + [int]$b[19]; $h = ([int]$b[20] -shl 24) + ([int]$b[21] -shl 16) + ([int]$b[22] -shl 8) + [int]$b[23]; return ,@($w, $h) }
function Files { @(Get-ChildItem $capDir -Filter '*.png' -ErrorAction SilentlyContinue) }
function Start-Eos([hashtable]$envs, [bool]$quiet = $true) {
  Cleanup-Eos; Start-Sleep 1
  foreach ($k in $envs.Keys) { Set-Item "env:$k" $envs[$k] }
  $env:EOSOS_CAPTURE_DIR = $capDir; if ($quiet) { $env:EOSOS_TEST_QUIET = '1' } else { Remove-Item env:EOSOS_TEST_QUIET -ErrorAction SilentlyContinue }
  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'; Start-Process $exe | Out-Null; Remove-Item env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS
  Start-Sleep 13
}

$capDir = Join-Path $env:TEMP ('eosos-cap-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
New-Item -ItemType Directory $capDir | Out-Null
$mc = Join-Path $env:LOCALAPPDATA 'EosOS\mission.json'
$bkMc = if (Test-Path $mc) { [System.IO.File]::ReadAllBytes($mc) } else { $null }
$wins = @()
try {
  Remove-Item $mc -ErrorAction SilentlyContinue
  Start-Eos @{}

  Write-Host 'A. impostazione'
  Check 'di base le scorciatoie sono spente' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('settings_all').then(function(a){return a.filter(function(e){return e.key==='capture.keys'})[0].value})") -eq 'off')
  Check 'spente: nessuna combinazione registrata' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_registered')") -eq 0)
  $r = Set-S 'capture.keys' "'ctrl_shift'"
  Check 'si sceglie Ctrl+Maiusc' ($r.ok -eq $true)
  Check 'le 3 combinazioni si registrano (o almeno 2)' (Wait-Until { (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_registered')") -ge 2 } 4)
  Check 'una scelta inventata e rifiutata' (((Set-S 'capture.keys' "'tutti'").ok) -eq $false)
  [void](Set-S 'capture.keys' "'off'")
  Check 'spente di nuovo: si liberano' (Wait-Until { (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_registered')") -eq 0 } 4)

  Write-Host 'B. schermo intero'
  # Tutti i processi eosos (il principale): le affinita' si leggono sul principale.
  $mainPid = [uint32](Get-CimInstance Win32_Process -Filter "Name='eosos.exe'" | Where-Object { $_.CommandLine -notmatch '--keyhook' } | Select-Object -First 1).ProcessId
  $before = [CT]::Affinities($mainPid)
  $cur = [System.Windows.Forms.Cursor]::Position
  $scr = [System.Windows.Forms.Screen]::FromPoint($cur).Bounds
  $s1 = Fire 'screen'
  Check 'lo scatto risponde con percorso e misura' (($null -ne $s1) -and ($s1.path -like "$capDir*"))
  Check 'il file PNG esiste' ((Test-Path $s1.path) -and ((Get-Item $s1.path).Length -gt 5000))
  $sz = Png-Size $s1.path
  Check 'la misura e quella del monitor del cursore' (($sz[0] -eq $scr.Width) -and ($sz[1] -eq $scr.Height)) ("PNG " + ($sz -join 'x') + " monitor " + $scr.Width + 'x' + $scr.Height)
  Check 'il nome e «Cattura schermo AAAA-MM-GG alle HH.MM.SS.png»' ((Split-Path $s1.path -Leaf) -match '^Cattura schermo \d{4}-\d{2}-\d{2} alle \d{2}\.\d{2}\.\d{2}( \(\d+\))?\.png$') (Split-Path $s1.path -Leaf)
  $bmp = [System.Drawing.Bitmap]::FromFile($s1.path)
  $colors = @{}; for ($i = 0; $i -lt 40; $i++) { $c = $bmp.GetPixel([int]($bmp.Width * ($i % 8 + 0.5) / 8), [int]($bmp.Height * ([math]::Floor($i / 8) + 0.5) / 5)); $colors[$c.ToArgb()] = 1 }
  $bmp.Dispose()
  Check 'non e un immagine nera o uniforme' ($colors.Count -ge 3) ("colori diversi: " + $colors.Count)
  Check 'dopo lo scatto le finestre di EosOS sono escluse dalla cattura come prima' (Wait-Until { [CT]::Affinities($mainPid) -eq $before } 4) ("prima: $before / dopo: " + [CT]::Affinities($mainPid))
  Start-Sleep -Seconds 1
  $s2 = Fire 'screen'
  Check 'due scatti di fila: due file diversi' (($s2.path -ne $s1.path) -and ((Files).Count -eq 2))

  Write-Host 'C. finestra in primo piano (magenta, mia)'
  $hf = Join-Path $env:TEMP "capwin-$([guid]::NewGuid().ToString('N')).txt"
  $pw = [System.Windows.Forms.Screen]::PrimaryScreen.WorkingArea
  $p = Start-Process powershell -ArgumentList @('-NoProfile', '-STA', '-ExecutionPolicy', 'Bypass', '-File', (Join-Path $PSScriptRoot 'snap-window.ps1'), '-Title', 'EosOS-Cap-Magenta', '-HandleFile', $hf, '-X', ($pw.Left + 300), '-Y', ($pw.Top + 200), '-W', 200, '-H', 150, '-Color', '#FF00FF', '-TopMost') -PassThru -WindowStyle Hidden
  $wins += @{ Pid = $p.Id; File = $hf }
  Check 'la finestra di prova compare' (Wait-Until { Test-Path $hf } 15)
  $h = [long](Get-Content $hf)
  # Una posizione dove la mia finestra e davvero la piu in alto (altre finestre dell utente potrebbero stare sopra): si provano alcune.
  $free = $false
  foreach ($c in @(@(20, 100), @(20, 400), @(20, 700), @(300, 200), @(1200, 100), @(900, 450))) {
    $x = $pw.Left + $c[0]; $y = $pw.Top + $c[1]
    [CT]::MoveTo($h, $x, $y)
    [CT]::ToTop($h); Start-Sleep -Milliseconds 500
    $fr = [CT]::Frame($h)
    $cx = [int](($fr[0] + $fr[2]) / 2); $cy = [int](($fr[1] + $fr[3]) / 2)
    $pts = @(@($cx, $cy), @(($fr[0] + 40), ($fr[1] + 40)), @(($fr[2] - 40), ($fr[1] + 40)), @(($fr[0] + 40), ($fr[3] - 40)), @(($fr[2] - 40), ($fr[3] - 40)))
    if (@($pts | Where-Object { -not [CT]::IsTopAt($h, [int]$_[0], [int]$_[1]) }).Count -eq 0) { $free = $true; break }
  }
  Write-Host ("  (posizione libera trovata: " + $free + ")")
  $fr = [CT]::Frame($h)
  $s3 = Fire 'window' $h
  Check 'lo scatto della finestra risponde' ($null -ne $s3)
  $sz = Png-Size $s3.path
  Check 'la misura e quella della parte visibile della finestra' (($sz[0] -eq ($fr[2] - $fr[0])) -and ($sz[1] -eq ($fr[3] - $fr[1]))) ("PNG " + ($sz -join 'x') + " finestra " + ($fr[2] - $fr[0]) + 'x' + ($fr[3] - $fr[1]))
  $bmp = [System.Drawing.Bitmap]::FromFile($s3.path)
  $mid = $bmp.GetPixel([int]($bmp.Width / 2), [int]($bmp.Height / 2)); $bmp.Dispose()
  if ($free) { Check 'il centro e magenta (la finestra giusta, niente altro sopra)' (($mid.R -gt 240) -and ($mid.G -lt 15) -and ($mid.B -gt 240)) ("colore " + $mid.R + ',' + $mid.G + ',' + $mid.B) } else { Write-Host '  (nessuna posizione libera: il colore non si controlla; la misura si)' }

  Write-Host 'D. selezione'
  $n0 = (Files).Count
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'region'})")
  Check 'il velo si apre' (Wait-Until { [CT]::Visible('EosOS Cattura') } 12)
  Start-Sleep -Milliseconds 800
  $ov = [CT]::Rect('EosOS Cattura')
  Check 'copre il monitor del cursore' (($ov[2] - $ov[0]) -eq $scr.Width -and ($ov[3] - $ov[1]) -eq $scr.Height) ("velo " + ($ov -join ','))
  $dpr = [double](Js 'capture' "window.devicePixelRatio")
  Check 'la pagina ha il suo suggerimento' ((Js 'capture' "(document.querySelector('.capture-hint')||{textContent:''}).textContent") -like 'Trascina*')
  $ev = "(function(){var r=document.querySelector('.capture');function ev(t,x,y){r.dispatchEvent(new PointerEvent(t,{clientX:x,clientY:y,button:0,pointerId:7,bubbles:true}))}"
  Js 'capture' ($ev + "ev('pointerdown',100,100);ev('pointermove',250,180);ev('pointermove',400,300);return 1})()") | Out-Null
  Start-Sleep -Milliseconds 300
  Check 'trascinando compare la misura' ((Js 'capture' "(document.querySelector('.capture-size')||{textContent:''}).textContent") -match '^\d+\s+\S+\s+\d+$') (Js 'capture' "(document.querySelector('.capture-size')||{textContent:''}).textContent")
  Js 'capture' ($ev + "ev('pointerup',400,300);return 1})()") | Out-Null
  Check 'al rilascio il velo sparisce' (Wait-Until { -not [CT]::Visible('EosOS Cattura') } 6)
  Check 'e nasce un file nuovo' (Wait-Until { (Files).Count -eq ($n0 + 1) } 8)
  $new = (Files | Sort-Object LastWriteTime | Select-Object -Last 1).FullName
  $sz = Png-Size $new
  $ew = [int][math]::Round(300 * $dpr); $eh = [int][math]::Round(200 * $dpr)
  Check 'la misura e quella scelta (300x200 CSS) in pixel fisici' (($sz[0] -eq $ew) -and ($sz[1] -eq $eh)) ("PNG " + ($sz -join 'x') + " atteso ${ew}x${eh} (scala $dpr)")

  $n1 = (Files).Count
  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'region'})")
  Check 'riaperto: Esc annulla' ((Wait-Until { [CT]::Visible('EosOS Cattura') } 12))
  Start-Sleep -Milliseconds 800
  Js 'capture' ($ev + "ev('pointerdown',50,50);ev('pointermove',200,200);window.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',bubbles:true}));return 1})()") | Out-Null
  Check 'il velo si chiude' (Wait-Until { -not [CT]::Visible('EosOS Cattura') } 6)
  Start-Sleep -Milliseconds 800
  Check 'e non nasce nessun file' ((Files).Count -eq $n1)

  [void](Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'region'})")
  Check 'riaperto: un clic senza trascinare annulla' ((Wait-Until { [CT]::Visible('EosOS Cattura') } 8))
  Start-Sleep -Milliseconds 800
  Js 'capture' ($ev + "ev('pointerdown',300,300);ev('pointerup',301,301);return 1})()") | Out-Null
  Check 'il velo si chiude' (Wait-Until { -not [CT]::Visible('EosOS Cattura') } 6)
  Start-Sleep -Milliseconds 800
  Check 'e non nasce nessun file' ((Files).Count -eq $n1)
  Check 'un tipo inventato non parte' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'boh'})") -eq $false)
  Check 'la finestra (tasto 5) non si avvia dal pannello' ((Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'window'})") -eq $false)

  Write-Host 'E. controllo accessi'
  Cleanup-Eos; Start-Sleep 1
  Start-Eos @{ EOSOS_TEST_STRICT = '1' }
  $n2 = (Files).Count
  Check 'dalla barra capture_run non parte' ((Js 'bar' "window.__TAURI_INTERNALS__.invoke('capture_run',{kind:'screen'}).then(function(r){return r},function(e){return false})") -ne $true)
  Check 'dal Launcher capture_region non scatta' ($null -eq (Js 'launcher' "window.__TAURI_INTERNALS__.invoke('capture_region',{x:10,y:10,w:100,h:100,dpr:1}).then(function(r){return r},function(e){return null})"))
  Start-Sleep -Seconds 2
  Check 'e non nasce nessun file' ((Files).Count -eq $n2)
  Remove-Item env:EOSOS_TEST_STRICT -ErrorAction SilentlyContinue

  Write-Host 'F. appunti (solo se contengono testo o nulla)'
  $clipText = $null; $clipOk = $true
  if ([System.Windows.Forms.Clipboard]::ContainsText()) { $clipText = [System.Windows.Forms.Clipboard]::GetText() } elseif ([System.Windows.Forms.Clipboard]::ContainsImage() -or [System.Windows.Forms.Clipboard]::ContainsFileDropList()) { $clipOk = $false }
  if ($clipOk) {
    try {
      Start-Eos @{} $false
      $s4 = Fire 'screen'
      Start-Sleep -Milliseconds 700
      $img = [System.Windows.Forms.Clipboard]::GetImage()
      Check 'negli appunti c e l immagine' ($null -ne $img)
      if ($img) { Check 'della misura dello scatto' (($img.Width -eq $s4.w) -and ($img.Height -eq $s4.h)) ("appunti " + $img.Width + 'x' + $img.Height + " scatto " + $s4.w + 'x' + $s4.h); $img.Dispose() }
    } finally {
      if ($null -ne $clipText -and $clipText -ne '') { [System.Windows.Forms.Clipboard]::SetText($clipText) } else { [System.Windows.Forms.Clipboard]::Clear() }
    }
  } else { Write-Host '  (negli appunti c e un immagine o dei file: non li tocco, salto)' }
}
finally {
  foreach ($x in $wins) { Stop-Process -Id $x.Pid -Force -ErrorAction SilentlyContinue; Remove-Item $x.File -ErrorAction SilentlyContinue }
  Cleanup-Eos
  Remove-Item env:EOSOS_TEST_STRICT, env:EOSOS_CAPTURE_DIR, env:EOSOS_TEST_QUIET -ErrorAction SilentlyContinue
  Remove-Item $capDir -Recurse -Force -ErrorAction SilentlyContinue
  Remove-Item $mc -ErrorAction SilentlyContinue
  if ($null -ne $bkMc) { [System.IO.File]::WriteAllBytes($mc, $bkMc) }
}
if ($script:fail -eq 0) { Write-Host 'TUTTO OK' -ForegroundColor Green } else { Write-Host "$($script:fail) controlli falliti" -ForegroundColor Red; exit 1 }
