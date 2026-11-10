# SPDX-License-Identifier: RPL-1.5
# T-01: DLL planting next to the installer (the Downloads-folder attack). Positive control first (the probe DLL
# must leave a marker when something loads it), then the setup and the uninstaller run from a folder full of
# planted DLLs. PASS = no marker from them. Usage: t01.ps1 <setup.exe> <probe.dll>
param([string]$Setup, [string]$Probe)
$ErrorActionPreference = "Stop"
$mark = Join-Path $env:TEMP "gdm-t01-hijack.txt"
$names = "version.dll", "uxtheme.dll", "cryptbase.dll", "userenv.dll", "profapi.dll", "propsys.dll", "dwmapi.dll",
  "wininet.dll", "secur32.dll", "netapi32.dll", "shfolder.dll", "setupapi.dll", "msimg32.dll", "textshaping.dll",
  "windowscodecs.dll", "ntmarta.dll", "apphelp.dll", "oleacc.dll", "riched20.dll", "comctl32.dll", "winmm.dll", "nsis_tauri_utils.dll"
Remove-Item $mark -ErrorAction SilentlyContinue
# positive control: rundll32 loads the probe -> marker must appear
Start-Process rundll32.exe -ArgumentList "`"$Probe`",DllMain" -Wait
$control = Test-Path $mark
"positive control (probe loaded by rundll32 leaves a marker): $control"
Remove-Item $mark -ErrorAction SilentlyContinue
$dir = Join-Path $env:TEMP ("gdm-t01-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory $dir | Out-Null
foreach ($n in $names) { Copy-Item $Probe (Join-Path $dir $n) }
Copy-Item $Setup (Join-Path $dir "GDM-setup.exe")
$p = Start-Process (Join-Path $dir "GDM-setup.exe") -ArgumentList "/S" -WorkingDirectory $dir -PassThru; $p.WaitForExit()
"setup exit=$($p.ExitCode); planted names: $($names.Count)"
Get-Process gdm -ErrorAction SilentlyContinue | Stop-Process -Force
$inst = Join-Path $env:LOCALAPPDATA "Galaxy Download Manager"
foreach ($n in $names) { Copy-Item $Probe (Join-Path $inst $n) -ErrorAction SilentlyContinue }   # also planted beside the uninstaller
$u = Start-Process (Join-Path $inst "uninstall.exe") -ArgumentList "/S" -PassThru; $u.WaitForExit(); Start-Sleep 3
$hits = if (Test-Path $mark) { Get-Content $mark } else { @() }
"markers after setup + uninstall: $($hits.Count) $($hits -join ' | ')"
Remove-Item -Recurse -Force $dir
Get-ChildItem $inst -ErrorAction SilentlyContinue | Where-Object { $names -contains $_.Name } | Remove-Item -Force
if ($control -and $hits.Count -eq 0) { "T-01 PASS" } else { "T-01 FAIL"; exit 1 }
