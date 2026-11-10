# SPDX-License-Identifier: RPL-1.5
# G7 I-02 end to end: gdm.exe /d ... over the per-user pipe (GDM not running => started in the tray), /p Startup
# refused, /p through a junction refused, `list` JSON, `start-queue`. Uses a throwaway data folder (GDM_DATA).
# usage: powershell -File tests/e2e/cli.ps1 <gdm.exe> <gdm-fixture.exe>
param([string]$Gdm, [string]$Fixture)
$ErrorActionPreference = "Stop"
$root = Join-Path $env:TEMP ("gdm-e2e-cli-" + $PID)
New-Item -ItemType Directory -Force $root | Out-Null
$env:GDM_DATA = Join-Path $root "data"
$dl = Join-Path $root "dl"; New-Item -ItemType Directory -Force $dl | Out-Null
$fail = 0
function Check($name, $ok, $detail) { if ($ok) { "PASS $name" } else { "FAIL $name :: $detail"; $script:fail++ } }
# Start-Process -Wait would also wait for the tray GDM a CLI call starts: wait for the CLI process only
function Invoke-Gdm([string[]]$a) {
  $psi = New-Object System.Diagnostics.ProcessStartInfo $Gdm
  $psi.Arguments = ($a | ForEach-Object { '"' + $_ + '"' }) -join ' '
  $psi.UseShellExecute = $false; $psi.RedirectStandardOutput = $true
  $p = [System.Diagnostics.Process]::Start($psi)
  $t = $p.StandardOutput.ReadToEndAsync()
  if (-not $p.WaitForExit(30000)) { $p.Kill(); return @{ ExitCode = -1; out = "timeout" } }
  if (-not $t.Wait(5000)) { return @{ ExitCode = $p.ExitCode; out = "stdout-not-closed" } }
  return @{ ExitCode = $p.ExitCode; out = $t.Result }
}
$fx = Start-Process -FilePath $Fixture -ArgumentList "18791" -PassThru -WindowStyle Hidden
Start-Sleep -Milliseconds 800
try {
  $r = Invoke-Gdm @("/d", "http://127.0.0.1:18791/p/3000000/5/0/cli-a.bin", "/p", $dl, "/n")
  $o1 = $r.out
  Check "/d /p /n while GDM is not running (starts it in the tray)" ($r.ExitCode -eq 0 -and $o1 -match '"ok":true') "exit $($r.ExitCode) $o1"
  $deadline = (Get-Date).AddSeconds(30)
  while (-not (Test-Path (Join-Path $dl "cli-a.bin")) -and (Get-Date) -lt $deadline) { Start-Sleep -Milliseconds 300 }
  Check "file downloaded into /p" (Test-Path (Join-Path $dl "cli-a.bin")) "missing"
  $startup = [Environment]::GetFolderPath("Startup")
  $r2 = Invoke-Gdm @("/d", "http://127.0.0.1:18791/p/1000/6/0/evil.exe", "/p", $startup, "/n")
  $o2 = $r2.out
  Check "/p Startup refused (I-02)" ($r2.ExitCode -eq 1 -and $o2 -match 'dangerous-folder') "exit $($r2.ExitCode) $o2"
  $j = Join-Path $root "junction"
  cmd /c mklink /J "$j" "$startup" | Out-Null
  $r3 = Invoke-Gdm @("/d", "http://127.0.0.1:18791/p/1000/7/0/evil2.exe", "/p", $j, "/n")
  $o3 = $r3.out
  Check "/p junction to Startup refused (I-02)" ($r3.ExitCode -eq 1 -and $o3 -match 'dangerous-folder|Reparse|junction') "exit $($r3.ExitCode) $o3"
  Check "nothing written to Startup" (-not (Test-Path (Join-Path $startup "evil.exe")) -and -not (Test-Path (Join-Path $startup "evil2.exe"))) "found"
  $r4 = Invoke-Gdm @("list")
  $o4 = $r4.out
  Check "list prints JSON with the job" ($r4.ExitCode -eq 0 -and $o4 -match 'cli-a.bin') "exit $($r4.ExitCode) $o4"
  $r5 = Invoke-Gdm @("/d", "http://127.0.0.1:18791/p/1000/8/0/later.bin", "/a")
  $o5 = $r5.out
  Check "/a adds to the main queue without starting" ($r5.ExitCode -eq 0 -and $o5 -match '"ok":true') "exit $($r5.ExitCode) $o5"
  $r6 = Invoke-Gdm @("/s")
  Check "/s starts the main queue" ($r6.ExitCode -eq 0) "exit $($r6.ExitCode)"
} finally {
  Get-Process gdm -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Resolve-Path $Gdm).Path } | Stop-Process -Force
  Stop-Process -Id $fx.Id -Force -ErrorAction SilentlyContinue
  if (Test-Path (Join-Path $root "junction")) { cmd /c rmdir (Join-Path $root "junction") | Out-Null }
}
"RESULT failures=$fail"
exit $fail
