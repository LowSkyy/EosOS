# Osservatore: gira in sottofondo e scrive su file ogni finestra che COMPARE, SPARISCE o CAMBIA (stile, posizione) fra quelle sottili e larghe (come una barra)
# o di EosOS, con l'ora, il processo e cosa c'e' sotto il mouse in quel momento. Serve a capire cosa disegna la barra «EosOS Dock» sopra il Dock: lo lasci
# aperto, usi il PC normalmente, e quando la barra compare e sparisce lo fermi (Ctrl+C). Poi basta dire «fatto»: il file lo leggo io.
#
# Uso:  powershell -File scripts\osserva-barra.ps1            (dura 15 minuti, oppure Ctrl+C)
#       powershell -File scripts\osserva-barra.ps1 -Minutes 40
# Non cambia nulla e non clicca niente. File: %TEMP%\osserva-barra.log
param([int]$Minutes = 15)
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text; using System.Collections.Generic;
public static class OB {
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
 public static string Describe(IntPtr h){ var t=new StringBuilder(120); var c=new StringBuilder(80); GetWindowText(h,t,120); GetClassName(h,c,80); R r; GetWindowRect(h,out r); uint pid; GetWindowThreadProcessId(h,out pid); return c+" | '"+t+"' | pid "+pid+" | rect "+r.L+","+r.T+","+r.Rt+","+r.B+" | stile 0x"+GetWindowLongPtr(h,-16).ToString("X")+" ex 0x"+GetWindowLongPtr(h,-20).ToString("X"); }
 public static string Under(){ P p; GetCursorPos(out p); IntPtr h=WindowFromPoint(p); if(h==IntPtr.Zero) return "(niente)"; return "mouse "+p.X+","+p.Y+" -> "+Describe(GetAncestor(h,2)); }
 // Finestre interessanti: visibili, e (sottili e larghe) oppure con "EosOS" nel titolo.
 public static Dictionary<long,string> Snapshot(){ var d=new Dictionary<long,string>(); EnumWindows((h,x)=>{ if(!IsWindowVisible(h)) return true; R r; GetWindowRect(h,out r); int w=r.Rt-r.L, hh=r.B-r.T; var t=new StringBuilder(120); GetWindowText(h,t,120); bool eos=t.ToString().Contains("EosOS"); if(eos || (hh>0 && hh<=140 && w>=300)){ d[h.ToInt64()]=Describe(h); } return true; },IntPtr.Zero); return d; }
}
'@
[void][OB]::SetProcessDpiAwarenessContext([IntPtr](-4))
$log = Join-Path $env:TEMP 'osserva-barra.log'
"Osservatore avviato $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss'), $Minutes minuti" | Set-Content $log -Encoding UTF8
Write-Host "Osservo per $Minutes minuti (Ctrl+C per fermare). Usa il PC normalmente e fai comparire la barra. Registro: $log"
$prev = [OB]::Snapshot()
foreach ($k in $prev.Keys) { "  [all'avvio] " + $prev[$k] | Add-Content $log -Encoding UTF8 }
$end = (Get-Date).AddMinutes($Minutes)
$lines = 0
while ((Get-Date) -lt $end) {
  Start-Sleep -Milliseconds 80
  $cur = [OB]::Snapshot()
  $now = Get-Date -Format 'HH:mm:ss.fff'
  foreach ($k in $cur.Keys) {
    if (-not $prev.ContainsKey($k)) { "$now COMPARSA  " + $cur[$k] + "   || " + [OB]::Under() | Add-Content $log -Encoding UTF8; $lines++ }
    elseif ($prev[$k] -ne $cur[$k]) { "$now CAMBIATA  prima: " + $prev[$k] + "`n             ora:   " + $cur[$k] + "   || " + [OB]::Under() | Add-Content $log -Encoding UTF8; $lines++ }
  }
  foreach ($k in $prev.Keys) { if (-not $cur.ContainsKey($k)) { "$now SPARITA   " + $prev[$k] | Add-Content $log -Encoding UTF8; $lines++ } }
  $prev = $cur
  if ($lines -gt 4000) { "troppe righe: mi fermo" | Add-Content $log -Encoding UTF8; break }
}
"Fine $(Get-Date -Format 'HH:mm:ss')" | Add-Content $log -Encoding UTF8
Write-Host "Finito. Registro in $log"
