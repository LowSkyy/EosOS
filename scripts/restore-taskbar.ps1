# Rimette in vista la barra delle applicazioni di Windows (su tutti i monitor) se EosOS l'ha lasciata nascosta, per esempio dopo aver
# fermato a forza TUTTI i processi eosos.exe: il ripristino lo fa un processo «watchdog» separato (un altro eosos.exe), che se viene
# ucciso insieme al principale non puo' piu' farlo. Sicuro da lanciare sempre: se la barra e' gia' visibile non cambia niente.
#
# Uso: powershell -File scripts\restore-taskbar.ps1
# In emergenza, con EosOS in esecuzione, c'e' anche la scorciatoia Ctrl+Alt+Maiusc+Esc.
Add-Type @'
using System; using System.Runtime.InteropServices; using System.Text;
public static class RTB { public delegate bool EP(IntPtr h, IntPtr l);
 [DllImport("user32.dll")] static extern bool EnumWindows(EP p, IntPtr l);
 [DllImport("user32.dll", CharSet=CharSet.Unicode)] static extern int GetClassName(IntPtr h, StringBuilder s, int n);
 [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr h);
 [DllImport("user32.dll")] static extern bool ShowWindow(IntPtr h, int c);
 public static int ShowAll(){ int n=0; EnumWindows((h,x)=>{ var c=new StringBuilder(80); GetClassName(h,c,80); var k=c.ToString(); if((k=="Shell_TrayWnd"||k=="Shell_SecondaryTrayWnd") && !IsWindowVisible(h)){ ShowWindow(h,5); n++; } return true; }, IntPtr.Zero); return n; } }
'@
$n = [RTB]::ShowAll()
if ($n -gt 0) { Write-Host "Barra delle applicazioni rimessa in vista ($n finestre)." } else { Write-Host 'La barra delle applicazioni era gia visibile.' }
