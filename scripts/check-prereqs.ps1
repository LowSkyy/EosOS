# Verifica i prerequisiti per sviluppare EosOS. Non installa nulla.
$ErrorActionPreference = 'Continue'
$env:Path += ";$env:USERPROFILE\.cargo\bin"
$fail = 0

function Check($name, [scriptblock]$test) {
    try {
        $r = & $test
        if ($r) { Write-Host ("[OK]     {0}: {1}" -f $name, $r) }
        else { Write-Host ("[MANCA]  {0}" -f $name) -ForegroundColor Red; $script:fail++ }
    } catch { Write-Host ("[MANCA]  {0}" -f $name) -ForegroundColor Red; $script:fail++ }
}

Check 'Rust (rustc)'  { (Get-Command rustc -ErrorAction Stop | Out-Null); rustc -V }
Check 'Cargo'         { (Get-Command cargo -ErrorAction Stop | Out-Null); cargo -V }
Check 'Toolchain MSVC' { $t = rustup show active-toolchain; if ($t -match 'msvc') { $t } }
Check 'Node.js'       { node -v }
Check 'npm'           { npm -v }
Check 'Git'           { git --version }
Check 'WebView2' {
    $k = 'HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}',
         'HKCU:\Software\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}'
    $k | ForEach-Object { (Get-ItemProperty $_ -ErrorAction SilentlyContinue).pv } | Where-Object { $_ } | Select-Object -First 1
}
Check 'VS Build Tools (C++)' {
    $vw = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
    if (Test-Path $vw) {
        & $vw -all -products * -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 -property installationPath | Select-Object -First 1
    }
}

if ($fail -gt 0) { Write-Host "`n$fail prerequisiti mancanti." -ForegroundColor Red; exit 1 }
Write-Host "`nTutti i prerequisiti sono presenti."
