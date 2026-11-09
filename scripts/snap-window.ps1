# Finestra di prova per test-winsnap.ps1: un modulo ridimensionabile (o a misura fissa con -Fixed) con titolo noto, che scrive il proprio
# handle in un file e resta aperto finche non si chiude (Stop-Process sul suo PID). Non fa altro.
param([string]$Title, [string]$HandleFile, [switch]$Fixed, [int]$X = 400, [int]$Y = 300, [int]$W = 700, [int]$H = 500, [string]$Color = '', [switch]$TopMost)
Add-Type -AssemblyName System.Windows.Forms
Add-Type -MemberDefinition '[DllImport("user32.dll")] public static extern bool SetProcessDpiAwarenessContext(IntPtr c); [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int c);' -Name Dpi -Namespace SnapWin
[void][SnapWin.Dpi]::SetProcessDpiAwarenessContext([IntPtr](-4))
$f = New-Object System.Windows.Forms.Form
$f.Text = $Title
if ($Color) { $f.BackColor = [System.Drawing.ColorTranslator]::FromHtml($Color) }
$f.StartPosition = 'Manual'
$f.Location = New-Object System.Drawing.Point($X, $Y)
$f.Size = New-Object System.Drawing.Size($W, $H)
if ($TopMost) { $f.TopMost = $true }
if ($Fixed) { $f.FormBorderStyle = 'FixedDialog'; $f.MaximizeBox = $false }
$f.Add_Shown({ $f.Activate(); [void][SnapWin.Dpi]::ShowWindow($f.Handle, 5); [System.IO.File]::WriteAllText($HandleFile, $f.Handle.ToInt64().ToString()) })
[System.Windows.Forms.Application]::Run($f)
