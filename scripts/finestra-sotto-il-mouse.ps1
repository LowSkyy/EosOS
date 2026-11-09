# Diagnostica: dice a quale finestra appartiene ciò che c'è SOTTO IL MOUSE (per esempio la barra azzurra «EosOS Dock» sopra il Dock).
# Uso: 1) lancia  powershell -File scripts\finestra-sotto-il-mouse.ps1   2) hai 8 secondi: fai comparire la barra e metti il mouse SOPRA la barra
#      (senza cliccare); 3) copia qui il testo che stampa. Non cambia nulla e non clicca niente.
param([int]$Seconds = 8)
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class WU {
 public delegate bool EP(IntPtr h, IntPtr l);
 [StructLayout(LayoutKind.Sequential)] public struct P { public int X, Y; }
 [StructLayout(LayoutKind.Sequential)] public struct R { public int L, T, Rt, B; }
 [DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);
 [DllImport("user32.dll")] static extern bool GetCursorPos(out P p);
 [DllImport("user32.dll")] static extern IntPtr WindowFromPoint(P p);
 [DllImport("user32.dll")] static extern IntPtr GetAncestor(IntPtr h, uint f);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr h, out R r);
 [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
 [DllImport("user32.dll")] static extern long GetWindowLongPtr(IntPtr h, int i);
 [DllImport("user32.dll")] static extern IntPtr GetWindow(IntPtr h, uint c);
 [DllImport("user32.dll")] static extern bool GetWindowDisplayAffinity(IntPtr h, out uint a);
 public static string Cursor(){ P p; GetCursorPos(out p); return p.X+","+p.Y; }
 static string Describe(IntPtr h){ var t=new StringBuilder(160); var c=new StringBuilder(100); GetWindowText(h,t,160); GetClassName(h,c,100); R r; GetWindowRect(h,out r); uint pid; GetWindowThreadProcessId(h,out pid); uint aff; GetWindowDisplayAffinity(h,out aff); IntPtr owner=GetWindow(h,4);
  return "hwnd="+h.ToInt64()+" classe="+c+" titolo='"+t+"' pid="+pid+" rect="+r.L+","+r.T+","+r.Rt+","+r.B+" visibile="+IsWindowVisible(h)+" stile=0x"+GetWindowLongPtr(h,-16).ToString("X")+" ex=0x"+GetWindowLongPtr(h,-20).ToString("X")+" affinita="+aff+" proprietario="+owner.ToInt64(); }
 public static string[] UnderCursor(){ P p; GetCursorPos(out p); var l=new List<string>(); IntPtr h=WindowFromPoint(p); l.Add("FINESTRA SOTTO IL MOUSE: "+Describe(h)); IntPtr root=GetAncestor(h,2); l.Add("FINESTRA PRINCIPALE:        "+Describe(root));
  l.Add("TUTTE LE FINESTRE VISIBILI CHE CONTENGONO IL PUNTO (dalla piu in alto):");
  EnumWindows((w,x)=>{ if(!IsWindowVisible(w)) return true; R r; GetWindowRect(w,out r); if(p.X>=r.L && p.X<r.Rt && p.Y>=r.T && p.Y<r.B){ l.Add("  "+Describe(w)); } return true; },IntPtr.Zero); return l.ToArray(); }
}
'@
[void][WU]::SetProcessDpiAwarenessContext([IntPtr](-4))
for ($i = $Seconds; $i -gt 0; $i--) { Write-Host "Metti il mouse sopra la barra... $i" -NoNewline; Write-Host "`r" -NoNewline; Start-Sleep 1 }
Write-Host ""
Write-Host ("Mouse a " + [WU]::Cursor())
$lines = [WU]::UnderCursor()
$lines | ForEach-Object { Write-Host $_ }
$pids = ($lines | ForEach-Object { if ($_ -match 'pid=(\d+)') { [int]$Matches[1] } } | Select-Object -Unique)
Write-Host ""
Write-Host "PROCESSI COINVOLTI:"
foreach ($p in $pids) { $pr = Get-Process -Id $p -ErrorAction SilentlyContinue; if ($pr) { Write-Host ("  pid " + $p + " = " + $pr.ProcessName + "  " + $pr.Path) } }
