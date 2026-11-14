# SPDX-License-Identifier: RPL-1.5
# Inside Windows Sandbox (the .wsb LogonCommand written by start.ps1): run qa.mjs with the host's Node and
# Playwright's Chromium (both mapped read-only), keep the console output, then delete what the run downloaded
# and shut the Sandbox down: its disk lives on the host's C: while it runs and is discarded when it closes.
$out = "C:\sbx\out"
$env:GDM_QA_CHROME = "C:\sbx\pw\chromium-1228\chrome-win64\chrome.exe"
# the test web server listens on the LAN address too (network-drop test): allow Node up front, or Windows
# Firewall's prompt covers the window the next real click aims at
netsh advfirewall firewall add rule name="gdm-qa node" dir=in action=allow program="C:\sbx\node\node.exe" enable=yes | Out-Null
& "C:\sbx\node\node.exe" "C:\sbx\in\qa\qa.mjs" *>&1 | Out-File -FilePath "$out\console.txt" -Encoding utf8
$code = $LASTEXITCODE
Get-ChildItem "$env:USERPROFILE\Downloads" -Force -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
Get-ChildItem $env:TEMP -Filter "gdm-qa-*" -ErrorAction SilentlyContinue | Remove-Item -Recurse -Force -ErrorAction SilentlyContinue
Set-Content -Path "$out\done.txt" -Value "done $code"
shutdown.exe /s /t 10
