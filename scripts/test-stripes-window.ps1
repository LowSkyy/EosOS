# Finestra di prova a strisce nere/bianche con del testo, in basso a tutta larghezza, che tiene
# conto della scala (DPI) dello schermo. Sta dietro al Dock e serve a verificare la sfocatura:
# test-dock-scales.ps1 la confronta con il contenuto sfocato della capsula.
# Si chiude con Stop-Process sul suo PID.
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c);' -Name Dpi -Namespace Stripes
[void][Stripes.Dpi]::SetProcessDpiAwarenessContext([IntPtr](-4))
$f = New-Object Windows.Forms.Form
$f.AutoScaleMode = 'None'; $f.FormBorderStyle = 'None'; $f.StartPosition = 'Manual'; $f.ShowInTaskbar = $false
$b = [Windows.Forms.Screen]::PrimaryScreen.Bounds        # pixel fisici: il processo e' per-monitor
$h = 520
$f.Bounds = New-Object Drawing.Rectangle $b.X, ($b.Bottom - $h), $b.Width, $h
$f.Add_Paint({ param($s, $e)
  $g = $e.Graphics; $w = 40
  for ($x = 0; $x -lt $f.Width; $x += $w) {
    $c = if ((($x / $w) % 2) -eq 0) { [Drawing.Color]::White } else { [Drawing.Color]::FromArgb(20, 20, 20) }
    $br = New-Object Drawing.SolidBrush $c; $g.FillRectangle($br, $x, 0, $w, $f.Height)
  }
  $font = New-Object Drawing.Font('Segoe UI', 64, [Drawing.FontStyle]::Bold)
  $g.DrawString('BLUR TEST', $font, [Drawing.Brushes]::OrangeRed, 420, 250)
})
$f.Add_Shown({ $f.Refresh() })
[Windows.Forms.Application]::Run($f)
