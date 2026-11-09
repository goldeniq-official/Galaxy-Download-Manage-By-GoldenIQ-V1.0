# SPIKE ONLY (G2): build + sign the MSIX. Usage: powershell -File build.ps1 -Version 0.1.0.0
param([string]$Version = "0.1.0.0")
$ErrorActionPreference = "Stop"
$sdk = "C:\Program Files (x86)\Windows Kits\10\bin\10.0.26100.0\x64"
$root = Split-Path $PSScriptRoot; $out = "$root\out\msix-$Version"
Remove-Item $out -Recurse -Force -ErrorAction SilentlyContinue; New-Item -ItemType Directory $out | Out-Null
(Get-Content "$PSScriptRoot\AppxManifest.template.xml" -Raw).Replace("@VERSION@", $Version) | Set-Content "$out\AppxManifest.xml" -Encoding utf8
Copy-Item "$root\bridge\target\release\gdm-bridge.exe" $out
foreach ($i in "StoreLogo","Square150x150Logo","Square44x44Logo") { Copy-Item "$root\..\..\src-tauri\icons\$i.png" $out }
$pkg = "$root\out\gdm-spike-$Version.msix"
& "$sdk\makeappx.exe" pack /o /d $out /p $pkg | Select-Object -Last 2
$cert = Get-ChildItem Cert:\CurrentUser\My | Where-Object Subject -eq "CN=GoldenIQ Spike" | Select-Object -First 1
if (-not $cert) { $cert = New-SelfSignedCertificate -Type Custom -Subject "CN=GoldenIQ Spike" -KeyUsage DigitalSignature `
  -CertStoreLocation Cert:\CurrentUser\My -TextExtension @("2.5.29.37={text}1.3.6.1.5.5.7.3.3","2.5.29.19={text}") -FriendlyName "GDM spike (throwaway)" }
& "$sdk\signtool.exe" sign /fd SHA256 /sha1 $cert.Thumbprint $pkg | Select-Object -Last 1
"PACKAGE $pkg"
