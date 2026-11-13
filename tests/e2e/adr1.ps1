# SPDX-License-Identifier: RPL-1.5
# G7 ADR-1 failure injection (one process: engine inside gdm.exe). Throwaway data folder (GDM_DATA).
#  A. crash the WebView: kill every msedgewebview2.exe descended from gdm.exe during a download
#     => gdm.exe must stay alive and the download must finish with the right bytes.
#  B. crash the core: kill gdm.exe during a download, start it again (--background)
#     => the job resumes by itself and finishes with the right bytes.
# Expected bytes = the same fixture URL fetched by curl. Afterwards IDM's "obsolete browser integration"
# dialogs (IDM 6.43 on this PC reacts to WebView2 launches) are answered No.
# usage: powershell -File tests/e2e/adr1.ps1 <gdm.exe> <gdm-fixture.exe>
param([string]$Gdm, [string]$Fixture)
$ErrorActionPreference = "Stop"
$root = Join-Path "D:\gdm-tmp" ("gdm-adr1-" + $PID)
New-Item -ItemType Directory -Force $root | Out-Null
$env:GDM_DATA = Join-Path $root "data"
$dl = Join-Path $root "dl"; New-Item -ItemType Directory -Force $dl | Out-Null
$fail = 0
function Check($name, $ok, $detail) { if ($ok) { "PASS $name" } else { "FAIL $name :: $detail"; $script:fail++ } }
function Invoke-Gdm([string[]]$a) {
  $psi = New-Object System.Diagnostics.ProcessStartInfo $Gdm
  $psi.Arguments = ($a | ForEach-Object { '"' + $_ + '"' }) -join ' '
  $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true
  $p = [System.Diagnostics.Process]::Start($psi)
  $t = $p.StandardOutput.ReadToEndAsync()
  if (-not $p.WaitForExit(30000)) { $p.Kill(); return "" }
  if (-not $t.Wait(5000)) { return "" }
  return $t.Result
}
function Descendants([int]$top) {
  $all = Get-CimInstance Win32_Process | Select-Object ProcessId, ParentProcessId, Name
  $set = @($top); $grew = $true
  while ($grew) { $grew = $false; foreach ($p in $all) { if ($set -contains $p.ParentProcessId -and -not ($set -contains $p.ProcessId)) { $set += $p.ProcessId; $grew = $true } } }
  $all | Where-Object { $set -contains $_.ProcessId -and $_.ProcessId -ne $top }
}
function Wait-Done([string]$name, [int]$secs) {
  $deadline = (Get-Date).AddSeconds($secs)
  while ((Get-Date) -lt $deadline) {
    $j = Invoke-Gdm @("list") | ConvertFrom-Json -ErrorAction SilentlyContinue
    $row = $j.jobs | Where-Object { $_.name -eq $name }
    if ($row -and ($row.state -eq "complete" -or $row.state -eq "unprotected")) { return $row }
    Start-Sleep -Milliseconds 500
  }
  return $null
}
$port = 18793
$fx = Start-Process -FilePath $Fixture -ArgumentList "$port" -PassThru -WindowStyle Hidden
Start-Sleep -Milliseconds 800
$gdmProc = $null
try {
  # ---- A: WebView crash ----
  $gdmProc = Start-Process -FilePath $Gdm -PassThru
  $deadline = (Get-Date).AddSeconds(20)
  while (-not (Descendants $gdmProc.Id | Where-Object { $_.Name -eq "msedgewebview2.exe" }) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 300 }
  $wv = @(Descendants $gdmProc.Id | Where-Object { $_.Name -eq "msedgewebview2.exe" })
  Check "WebView2 runs under gdm.exe" ($wv.Count -gt 0) "none"
  $urlA = "http://127.0.0.1:$port/file?size=157286400&seed=31&trate=6000000"
  $o = Invoke-Gdm @("/d", $urlA, "/p", $dl, "/f", "adr1-a.bin", "/n")
  Check "download A started" ($o -match '"ok":true') $o
  Start-Sleep -Seconds 4
  $wv = @(Descendants $gdmProc.Id | Where-Object { $_.Name -eq "msedgewebview2.exe" })
  foreach ($p in $wv) { Stop-Process -Id $p.ProcessId -Force -ErrorAction SilentlyContinue }
  "killed $($wv.Count) WebView2 processes at $(Get-Date -Format o)"
  Start-Sleep -Seconds 2
  Check "gdm.exe alive after the WebView crash" (-not $gdmProc.HasExited) "exited"
  # the window comes back: new WebView2 processes under gdm.exe (ProcessFailed => window rebuilt)
  $deadline = (Get-Date).AddSeconds(20); $back = @()
  while ($back.Count -eq 0 -and (Get-Date) -lt $deadline) {
    Start-Sleep -Milliseconds 500
    $back = @(Descendants $gdmProc.Id | Where-Object { $_.Name -eq "msedgewebview2.exe" -and -not ($wv.ProcessId -contains $_.ProcessId) })
  }
  $wvlog = Join-Path $env:GDM_DATA "webview.log"
  Check "WebView rebuilt after the crash (new WebView2 processes)" ($back.Count -gt 0) ("no new WebView2 process; log: " + $(if (Test-Path $wvlog) { Get-Content $wvlog -Raw } else { "none" }))
  $rowA = Wait-Done "adr1-a.bin" 120
  Check "download A finished after the WebView crash" ($null -ne $rowA) "not complete"
  & curl.exe -sS -o (Join-Path $root "ref-a.bin") ($urlA -replace "&trate=6000000", "")
  $okA = (Test-Path (Join-Path $dl "adr1-a.bin")) -and ((Get-FileHash (Join-Path $dl "adr1-a.bin")).Hash -eq (Get-FileHash (Join-Path $root "ref-a.bin")).Hash)
  Check "download A bytes correct" $okA "hash differs"

  # ---- B: core crash ----
  $urlB = "http://127.0.0.1:$port/file?size=157286400&seed=32&trate=6000000"
  $o = Invoke-Gdm @("/d", $urlB, "/p", $dl, "/f", "adr1-b.bin", "/n")
  Check "download B started" ($o -match '"ok":true') $o
  Start-Sleep -Seconds 8
  Get-Process gdm -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Resolve-Path $Gdm).Path } | Stop-Process -Force
  "killed gdm.exe at $(Get-Date -Format o)"
  Start-Sleep -Seconds 2
  $gdmProc = Start-Process -FilePath $Gdm -ArgumentList "--background" -PassThru
  $rowB = Wait-Done "adr1-b.bin" 180
  Check "download B resumed by itself and finished after the core crash" ($null -ne $rowB) "not complete"
  & curl.exe -sS -o (Join-Path $root "ref-b.bin") ($urlB -replace "&trate=6000000", "")
  $okB = (Test-Path (Join-Path $dl "adr1-b.bin")) -and ((Get-FileHash (Join-Path $dl "adr1-b.bin")).Hash -eq (Get-FileHash (Join-Path $root "ref-b.bin")).Hash)
  Check "download B bytes correct" $okB "hash differs"
} finally {
  Get-Process gdm -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Resolve-Path $Gdm).Path } | Stop-Process -Force
  Stop-Process -Id $fx.Id -Force -ErrorAction SilentlyContinue
  # answer IDM's integration dialogs with No (WM_COMMAND IDNO=7)
  Add-Type -Namespace W -Name U -MemberDefinition '[DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern System.IntPtr FindWindowW(string c, string t); [DllImport("user32.dll")] public static extern System.IntPtr SendMessageW(System.IntPtr h, uint m, System.IntPtr w, System.IntPtr l);' -ErrorAction SilentlyContinue
  $n = 0
  for ($i = 0; $i -lt 60; $i++) { $h = [W.U]::FindWindowW("#32770", "Internet Download Manager"); if ($h -eq [System.IntPtr]::Zero) { break }; [void][W.U]::SendMessageW($h, 0x111, [System.IntPtr]7, [System.IntPtr]::Zero); $n++; Start-Sleep -Milliseconds 200 }
  "IDM dialogs answered No: $n"
  Remove-Item -Recurse -Force $root -ErrorAction SilentlyContinue
}
"RESULT failures=$fail"
exit $fail
