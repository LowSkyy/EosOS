# Gira DENTRO Windows Sandbox (lanciato da sandbox\EosOS-update.wsb): prova un AGGIORNAMENTO VERO, da 0.1.0 a 0.1.1.
# 1) installa la 0.1.0 (costruita con il feed su http://127.0.0.1:18080, solo per questa prova) in silenzio;
# 2) serve il feed e la 0.1.1 FIRMATA con la chiave vera da un piccolo server locale (HttpListener);
# 3) avvia EosOS 0.1.0 e, dalla finestra Impostazioni (con la porta di debug di WebView2), chiama update_check e
#    update_install come farebbe il pulsante; il plugin verifica la firma con la chiave pubblica incorporata;
# 4) controlla che dopo l'installazione il programma installato sia la 0.1.1 e che parta; poi disinstalla.
# Scrive tutto in C:\EosOS-out\risultati-update.txt. L'installazione resta dentro la Sandbox.
$out = 'C:\EosOS-out\risultati-update.txt'
function Log($t) { Add-Content -Path $out -Value $t -Encoding utf8 }
function Check($l, $c, $d = '') { Log ($(if ($c) { '[OK]      ' } else { '[FALLITO] ' }) + "$l $d") }
Start-Sleep 20
Set-Content -Path $out -Value "Prova di aggiornamento in Windows Sandbox - $(Get-Date -Format 'yyyy-MM-dd HH:mm:ss')" -Encoding utf8
Log "OS: $((Get-CimInstance Win32_OperatingSystem).Caption) $((Get-CimInstance Win32_OperatingSystem).Version); utente $env:USERNAME"

# --- CDP minimo (WebSocket) per chiamare i comandi dalla pagina Impostazioni ---
function Cdp-Eval([string]$wsUrl, [string]$expr, [int]$timeoutSec = 30, [bool]$wait = $true) {
  $ws = New-Object System.Net.WebSockets.ClientWebSocket
  $cts = New-Object System.Threading.CancellationTokenSource ($timeoutSec * 1000)
  $ws.ConnectAsync([Uri]$wsUrl, $cts.Token).Wait()
  $msg = @{ id = 1; method = 'Runtime.evaluate'; params = @{ expression = $expr; awaitPromise = $true; returnByValue = $true } } | ConvertTo-Json -Depth 5 -Compress
  $bytes = [Text.Encoding]::UTF8.GetBytes($msg)
  $ws.SendAsync([ArraySegment[byte]]$bytes, 'Text', $true, $cts.Token).Wait()
  if (-not $wait) { return $null }
  $buf = New-Object byte[] 65536; $sb = New-Object Text.StringBuilder
  try {
    while ($true) {
      $r = $ws.ReceiveAsync([ArraySegment[byte]]$buf, $cts.Token); $r.Wait()
      [void]$sb.Append([Text.Encoding]::UTF8.GetString($buf, 0, $r.Result.Count))
      if ($r.Result.EndOfMessage) { $o = $sb.ToString() | ConvertFrom-Json; if ($o.id -eq 1) { return $o.result.result.value }; [void]$sb.Clear() }
    }
  } catch { return "ERRCDP: $_" } finally { try { $ws.Dispose() } catch {} }
}
$script:lastErr = ''
$script:targets = ''
function Settings-Ws {
  foreach ($h in 'localhost', '127.0.0.1') {
    try {
      $t = @(Invoke-RestMethod "http://${h}:9222/json" -TimeoutSec 5)
      $script:targets = ($t | ForEach-Object { "$($_.type) $($_.url)" }) -join ' ; '
      $w = ($t | Where-Object { $_.url -like '*view=settings*' } | Select-Object -First 1).webSocketDebuggerUrl
      if ($w) { return $w }
    } catch { $script:lastErr = "$h : $_" }
  }
  $null
}
function Installed-Version($exe) { (Get-Item $exe).VersionInfo.ProductVersion }

try {
  $setupA = Get-ChildItem 'C:\EosOS\EosOS_A_*-setup.exe' | Select-Object -First 1
  $feedDir = 'C:\EosOS\feed'
  $setupB = Get-ChildItem "$feedDir\EosOS_0.1.1_x64-setup.exe" | Select-Object -First 1
  $sigB = (Get-Content "$($setupB.FullName).sig" -Raw).Trim()
  Check 'file di prova presenti (setup 0.1.0, setup 0.1.1 e firma)' (($null -ne $setupA) -and ($null -ne $setupB) -and ($sigB.Length -gt 100))

  $p = Start-Process $setupA.FullName -ArgumentList '/S' -PassThru -Wait
  Check 'installazione silenziosa della 0.1.0' ($p.ExitCode -eq 0) "codice $($p.ExitCode)"
  $exe = Join-Path $env:LOCALAPPDATA 'EosOS\eosos.exe'
  Check 'eosos.exe installato' (Test-Path $exe) $exe
  $v0 = Installed-Version $exe
  Check 'versione installata 0.1.0' ($v0 -eq '0.1.0') $v0

  # Feed locale: latest.json costruito qui con la firma vera e l'indirizzo locale.
  $entry = @{ signature = $sigB; url = 'http://127.0.0.1:18080/EosOS_0.1.1_x64-setup.exe' }
  $feed = [ordered]@{ version = '0.1.1'; notes = 'Versione di prova'; pub_date = '2026-11-08T12:00:00Z'; platforms = [ordered]@{ 'windows-x86_64' = $entry; 'windows-x86_64-nsis' = $entry } }
  $latest = ($feed | ConvertTo-Json -Depth 5)
  $listener = New-Object System.Net.HttpListener
  $listener.Prefixes.Add('http://127.0.0.1:18080/')
  $listener.Start()
  # Il listener principale gira in un thread dedicato (runspace) per non bloccare lo script.
  $rs = [runspacefactory]::CreateRunspace(); $rs.Open()
  $ps = [powershell]::Create(); $ps.Runspace = $rs
  [void]$ps.AddScript({
    param($listener, $latest, $setupPath)
    while ($listener.IsListening) {
      try { $ctx = $listener.GetContext() } catch { break }
      $path = $ctx.Request.Url.AbsolutePath
      if ($path -eq '/latest.json') { $b = (New-Object Text.UTF8Encoding($false)).GetBytes($latest); $ctx.Response.ContentType = 'application/json' }
      elseif ($path -like '*setup.exe') { $b = [IO.File]::ReadAllBytes($setupPath); $ctx.Response.ContentType = 'application/octet-stream' }
      else { $ctx.Response.StatusCode = 404; $b = @() }
      $ctx.Response.ContentLength64 = $b.Length
      if ($b.Length) { $ctx.Response.OutputStream.Write($b, 0, $b.Length) }
      $ctx.Response.Close()
    }
  }).AddArgument($listener).AddArgument($latest).AddArgument($setupB.FullName)
  $handle = $ps.BeginInvoke()
  Check 'server del feed locale avviato' ($listener.IsListening)

  $env:WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS = '--remote-debugging-port=9222'
  Start-Process $exe
  Start-Sleep 25
  Check 'EosOS 0.1.0 in esecuzione' (@(Get-Process eosos -ErrorAction SilentlyContinue).Count -ge 1)
  $ws = $null
  for ($i = 0; $i -lt 10 -and -not $ws; $i++) { try { $ws = Settings-Ws } catch {}; if (-not $ws) { Start-Sleep 2 } }
  Check 'pagina Impostazioni raggiungibile' ([bool]$ws) "ultimo errore: $script:lastErr; pagine viste: $script:targets"
  if (-not $ws) { throw 'senza la pagina Impostazioni non si puo continuare' }

  $st = Cdp-Eval $ws "window.__TAURI_INTERNALS__.invoke('update_check').then(function(r){return JSON.stringify(r)},function(e){return 'ERR:'+e})"
  Log "update_check -> $st"
  Check 'update_check: disponibile la 0.1.1' (($st -like '*"state":"available"*') -and ($st -like '*0.1.1*'))

  # Installazione: l'app si chiude durante l'aggiornamento, quindi non si aspetta la risposta.
  [void](Cdp-Eval $ws "window.__TAURI_INTERNALS__.invoke('update_install')" 10 $false)
  $done = $false
  for ($i = 0; $i -lt 90 -and -not $done; $i++) { Start-Sleep 2; $done = (Test-Path $exe) -and ((Installed-Version $exe) -eq '0.1.1') }
  $v1 = if (Test-Path $exe) { Installed-Version $exe } else { '(manca)' }
  Check 'dopo update_install il programma installato e la 0.1.1 (firma verificata e installatore eseguito)' ($v1 -eq '0.1.1') $v1
  Start-Sleep 10
  $alive = @(Get-Process eosos -ErrorAction SilentlyContinue)
  if ($alive.Count -eq 0 -and (Test-Path $exe)) { Start-Process $exe; Start-Sleep 20; $alive = @(Get-Process eosos -ErrorAction SilentlyContinue) }
  Check 'EosOS 0.1.1 parte e resta in vita' ($alive.Count -ge 1)

  Get-Process eosos -ErrorAction SilentlyContinue | Stop-Process -Force
  Start-Sleep 3
  $uninst = Get-ChildItem (Join-Path $env:LOCALAPPDATA 'EosOS') -Filter 'uninstall*.exe' -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($uninst) { $u = Start-Process $uninst.FullName -ArgumentList '/S' -PassThru -Wait; Check 'disinstallazione silenziosa' ($u.ExitCode -eq 0) "codice $($u.ExitCode)" }
  $listener.Stop()
} catch { Log "ERRORE: $_" }
Log "`nPROVA FINITA."
