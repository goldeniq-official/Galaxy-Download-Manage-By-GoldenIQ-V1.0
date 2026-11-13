# SPDX-License-Identifier: RPL-1.5
# Prepare a throwaway GDM data folder with real jobs for tests/e2e/webview2.mjs: settings (onboarded, language)
# + finished, paused and queued downloads made through the CLI (G7 I-02) from the local fixture.
# usage: powershell -File tests/e2e/seed-data.ps1 <install dir> <data dir> <gdm-fixture.exe> <km|en>
param([string]$Inst, [string]$Data, [string]$Fixture, [string]$Lang = "km")
$ErrorActionPreference = "Stop"
New-Item -ItemType Directory -Force $Data | Out-Null
$dl = Join-Path $Data "dl"; New-Item -ItemType Directory -Force $dl | Out-Null
$s = @{ lang = $Lang; onboarded = $true; download_dir = $dl } | ConvertTo-Json
[IO.File]::WriteAllText((Join-Path $Data "settings.json"), $s)
$env:GDM_DATA = $Data
$fx = Start-Process -FilePath $Fixture -ArgumentList "18794" -PassThru -WindowStyle Hidden
Start-Sleep -Milliseconds 800
$gdm = Join-Path $Inst "gdm.exe"
function Invoke-Gdm([string[]]$a) { $p = Start-Process -FilePath $gdm -ArgumentList $a -PassThru -WindowStyle Hidden; [void]$p.WaitForExit(30000) }
try {
  Invoke-Gdm @("/d", "http://127.0.0.1:18794/p/2000000/1/0/setup-installer.exe", "/n")
  Invoke-Gdm @("/d", "http://127.0.0.1:18794/p/3000000/2/0/report-2026.pdf", "/n")
  Invoke-Gdm @("/d", "http://127.0.0.1:18794/p/1500000/3/0/photos.zip", "/n")
  Invoke-Gdm @("/d", "http://127.0.0.1:18794/p/9000000/4/0/later.iso", "/a")
  Start-Sleep -Seconds 6
} finally {
  Get-Process gdm -ErrorAction SilentlyContinue | Where-Object { $_.Path -eq (Resolve-Path $gdm).Path } | Stop-Process -Force
  Stop-Process -Id $fx.Id -Force -ErrorAction SilentlyContinue
}
"seeded $Data"
