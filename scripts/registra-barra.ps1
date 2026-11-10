# Registratore: guarda lo schermo (la fascia in basso, dove sta il Dock) e, appena compare una banda ORIZZONTALE azzurra chiara larga e bassa (come la barra
# «EosOS Dock» della foto), salva un'immagine della fascia e ELENCA TUTTE LE FINESTRE che stanno in quel punto (anche trasparenti, anche di altri programmi:
# overlay di NVIDIA, Medal, Discord, Steam...), con il programma a cui appartengono. Cosi' si capisce chi la disegna.
#
# Uso:  powershell -File scripts\registra-barra.ps1                    (dura 20 minuti, si ferma da solo dopo 3 rilevamenti, oppure Ctrl+C)
#       powershell -File scripts\registra-barra.ps1 -Minutes 40 -Test   (-Test: la prova con una finestra mia azzurra, per vedere che funziona)
# Non cambia nulla e non clicca niente. File: %TEMP%\registra-barra-N.png e il testo qui sotto, anche in %TEMP%\registra-barra.log
param([int]$Minutes = 20, [switch]$Test)
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -ReferencedAssemblies System.Drawing -TypeDefinition @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic; using System.Drawing; using System.Drawing.Imaging;
public static class RB {
 public delegate bool EP(IntPtr h, IntPtr l);
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
 [DllImport("user32.dll")] static extern long GetWindowLongPtr(IntPtr h, int i);
 [DllImport("user32.dll")] static extern bool GetWindowDisplayAffinity(IntPtr h, out uint a);
 // Cerca una banda: una riga (ogni 2 px in verticale) con almeno minRun pixel azzurri consecutivi (campionati ogni 3 px) e alta almeno minRows righe. Restituisce x,y,w,h o null.
 public static int[] FindBand(Bitmap b, int minRunPx, int minHeightPx){
  var data=b.LockBits(new Rectangle(0,0,b.Width,b.Height), ImageLockMode.ReadOnly, PixelFormat.Format32bppArgb); int stride=data.Stride; byte[] px=new byte[stride*b.Height]; Marshal.Copy(data.Scan0,px,0,px.Length); b.UnlockBits(data);
  int rows=0,startY=0,runX=0,runW=0; int[] res=null;
  for(int y=0;y<b.Height;y+=2){ int best=0,bx=0,cur=0,cx=0; for(int x=0;x<b.Width;x+=3){ int i=y*stride+x*4; int B=px[i],G=px[i+1],Rr=px[i+2]; bool lb = Rr>=140&&Rr<=220&&G>=195&&G<=245&&B>=225&&B<=255&&(B-Rr)>=25; if(lb){ if(cur==0) cx=x; cur+=3; if(cur>best){best=cur;bx=cx;} } else cur=0; }
   if(best>=minRunPx){ if(rows==0){startY=y; runX=bx; runW=best;} rows+=2; if(best>runW){runW=best;} } else { if(rows>=minHeightPx){ res=new int[]{runX,startY,runW,rows}; break; } rows=0; }
  }
  if(res==null && rows>=minHeightPx) res=new int[]{runX,startY,runW,rows};
  return res; }
 static string D(IntPtr h){ var t=new StringBuilder(120); var c=new StringBuilder(80); GetWindowText(h,t,120); GetClassName(h,c,80); R r; GetWindowRect(h,out r); uint pid; GetWindowThreadProcessId(h,out pid); uint aff; GetWindowDisplayAffinity(h,out aff); return "classe="+c+" titolo='"+t+"' pid="+pid+" rect="+r.L+","+r.T+","+r.Rt+","+r.B+" stile=0x"+GetWindowLongPtr(h,-16).ToString("X")+" ex=0x"+GetWindowLongPtr(h,-20).ToString("X")+" affinita="+aff; }
 public static string[] WindowsAt(int x, int y){ var l=new List<string>(); EnumWindows((h,z)=>{ if(!IsWindowVisible(h)) return true; R r; GetWindowRect(h,out r); if(x>=r.L&&x<r.Rt&&y>=r.T&&y<r.B) l.Add(D(h)); return true; },IntPtr.Zero); return l.ToArray(); }
}
'@
[void][RB]::SetProcessDpiAwarenessContext([IntPtr](-4))
$log = Join-Path $env:TEMP 'registra-barra.log'
"Registratore avviato $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" | Set-Content $log -Encoding UTF8
$probe = $null
if ($Test) {
  $probe = New-Object System.Windows.Forms.Form; $probe.FormBorderStyle = 'None'; $probe.BackColor = [System.Drawing.Color]::FromArgb(176, 220, 244); $probe.StartPosition = 'Manual'
  $probe.Location = New-Object System.Drawing.Point(300, 800); $probe.Size = New-Object System.Drawing.Size(900, 44); $probe.TopMost = $true; $probe.ShowInTaskbar = $false; $probe.Text = 'Barra di prova'
  $probe.Show(); for ($k = 0; $k -lt 8; $k++) { [System.Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 100 }
  Write-Host 'Prova: una finestra azzurra mia a 300,800 (900x44).'
}
$vs = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bandTop = [math]::Max($vs.Top, $vs.Bottom - 420)
Write-Host "Guardo la fascia in basso ($($vs.Left),$bandTop $($vs.Width)x$($vs.Bottom - $bandTop)) per $Minutes minuti. Usa il PC e fai comparire la barra. Ctrl+C per fermare. Registro: $log"
$end = (Get-Date).AddMinutes($Minutes)
$found = 0
$lastSave = [datetime]::MinValue
try {
  while ((Get-Date) -lt $end -and $found -lt 3) {
    $bmp = New-Object System.Drawing.Bitmap $vs.Width, ($vs.Bottom - $bandTop)
    $g = [System.Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($vs.Left, $bandTop, 0, 0, $bmp.Size); $g.Dispose()
    $band = [RB]::FindBand($bmp, 450, 20)
    if ($band -and ((Get-Date) - $lastSave).TotalSeconds -gt 2) {
      $found++; $lastSave = Get-Date
      $cx = $vs.Left + $band[0] + [int]($band[2] / 2); $cy = $bandTop + $band[1] + [int]($band[3] / 2)
      $png = Join-Path $env:TEMP "registra-barra-$found.png"; $bmp.Save($png)
      $msg = "$(Get-Date -Format 'HH:mm:ss.fff') BARRA TROVATA: x=$($vs.Left + $band[0]) y=$($bandTop + $band[1]) larga $($band[2]) alta $($band[3]) -> immagine $png`nFINESTRE IN QUEL PUNTO ($cx,$cy), dalla piu in alto:"
      $msg | Tee-Object -FilePath $log -Append | Write-Host
      [RB]::WindowsAt($cx, $cy) | ForEach-Object {
        $pidm = if ($_ -match 'pid=(\d+)') { [int]$Matches[1] } else { 0 }
        $pn = (Get-Process -Id $pidm -ErrorAction SilentlyContinue).ProcessName
        ("   " + $_ + "  [programma: $pn]") | Tee-Object -FilePath $log -Append | Write-Host
      }
    }
    $bmp.Dispose()
    if ($probe) { [System.Windows.Forms.Application]::DoEvents() }
    Start-Sleep -Milliseconds 120
  }
}
finally { if ($probe) { $probe.Close() } }
"Fine $(Get-Date -Format 'HH:mm:ss'), rilevamenti: $found" | Tee-Object -FilePath $log -Append | Write-Host
