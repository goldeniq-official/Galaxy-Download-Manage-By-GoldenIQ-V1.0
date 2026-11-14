# SPDX-License-Identifier: RPL-1.5
# Host side of the pre-release QA in Windows Sandbox. Stages a FRESH build (never a stale one: built.txt names
# the commit), writes gdm-qa.wsb and opens the Sandbox; tests/sandbox/qa.mjs runs inside; results land in
# <stage>\out (report.md, results.txt, shots\). Keep the Sandbox window visible (not minimized) while it runs:
# screenshots of a hidden session come out blank and the report marks them.
# usage: powershell -File tests/sandbox/start.ps1 -Setup <GDM-Setup.exe> -Nsis <NSIS setup.exe> [-Stage D:\gdm-sbx]
param([Parameter(Mandatory)][string]$Setup, [Parameter(Mandatory)][string]$Nsis, [string]$Stage = "D:\gdm-sbx")
$ErrorActionPreference = "Stop"
$repo = (Resolve-Path "$PSScriptRoot\..\..").Path
$in = Join-Path $Stage "in"; $out = Join-Path $Stage ("out-" + (Get-Date -Format "yyyyMMdd-HHmmss"))
if (Get-Process WindowsSandbox, WindowsSandboxClient -ErrorAction SilentlyContinue) { throw "a Windows Sandbox is already open (only one can run): close it first" }
New-Item -ItemType Directory -Force $in, $out, "$in\qa" | Out-Null
Copy-Item $Setup "$in\gdm-setup.exe" -Force
Copy-Item $Nsis "$in\setup.exe" -Force
Copy-Item "$PSScriptRoot\qa.mjs", "$PSScriptRoot\win.ps1", "$PSScriptRoot\run.ps1" "$in\qa" -Force
if (Test-Path "$in\ext") { Remove-Item "$in\ext" -Recurse -Force }
Copy-Item "$repo\extension\dist\chrome" "$in\ext" -Recurse
# what was built: the installers' own times next to the commit checked out now (a build older than HEAD is stale)
Push-Location $repo
$head = git log -1 --format="%h %ci"
$dirty = (git status --short) -join "; "
Pop-Location
"setup built $((Get-Item $Setup).LastWriteTime.ToString('s')), HEAD $head, uncommitted: $dirty" | Set-Content "$in\built.txt" -Encoding utf8
$node = Split-Path (Get-Command node).Source
$pw = Join-Path $env:LOCALAPPDATA "ms-playwright"
if (-not (Test-Path "$pw\chromium-1228\chrome-win64\chrome.exe")) { throw "Playwright Chromium 1228 missing (pnpm exec playwright install chromium)" }
$map = { param($h, $s, $ro) "    <MappedFolder><HostFolder>$h</HostFolder><SandboxFolder>$s</SandboxFolder><ReadOnly>$ro</ReadOnly></MappedFolder>" }
$wsb = @"
<Configuration>
  <Networking>Enable</Networking>
  <MemoryInMB>6144</MemoryInMB>
  <MappedFolders>
$(& $map $in "C:\sbx\in" "true")
$(& $map $out "C:\sbx\out" "false")
$(& $map $node "C:\sbx\node" "true")
$(& $map $pw "C:\sbx\pw" "true")
  </MappedFolders>
  <LogonCommand><Command>powershell.exe -NoProfile -ExecutionPolicy Bypass -File C:\sbx\in\qa\run.ps1</Command></LogonCommand>
</Configuration>
"@
$wsbPath = Join-Path $Stage "gdm-qa.wsb"
Set-Content $wsbPath $wsb -Encoding utf8
Start-Process $wsbPath
"results: $out"
