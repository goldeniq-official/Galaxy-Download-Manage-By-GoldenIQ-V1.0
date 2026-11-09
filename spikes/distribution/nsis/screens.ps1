# SPIKE ONLY (G2/S9): walk the installer + uninstaller with Enter, capture each page.
param([string]$Out)
Add-Type -AssemblyName System.Drawing, System.Windows.Forms
Add-Type @"
using System; using System.Runtime.InteropServices;
public static class W {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern int GetDpiForWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L, T, R, B; }
}
"@
[W]::SetProcessDPIAware() | Out-Null  # else GetWindowRect is scaled at 125%
New-Item -ItemType Directory -Force $Out | Out-Null
function Shot($proc, $name) {
  Start-Sleep -Milliseconds 1200
  $h = (Get-Process -Id $proc.Id -ErrorAction SilentlyContinue).MainWindowHandle
  if (-not $h -or $h -eq 0) { "  $name : no window"; return }
  [W]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 300
  $r = New-Object W+RECT; [W]::GetWindowRect($h, [ref]$r) | Out-Null
  $bmp = New-Object Drawing.Bitmap ($r.R - $r.L), ($r.B - $r.T)
  [Drawing.Graphics]::FromImage($bmp).CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
  $bmp.Save("$Out\$name.png"); "  $name : $($r.R - $r.L)x$($r.B - $r.T) dpi=$([W]::GetDpiForWindow($h))"
}
function Enter($proc) { $h = (Get-Process -Id $proc.Id).MainWindowHandle; [W]::SetForegroundWindow($h) | Out-Null; Start-Sleep -Milliseconds 300; [Windows.Forms.SendKeys]::SendWait("{ENTER}") }

$root = Split-Path $PSScriptRoot
$p = Start-Process "$root\out\GDM-Spike-Setup-0.1.0.exe" -PassThru
Shot $p "01-language"; Enter $p
Shot $p "02-welcome"; Enter $p
Shot $p "03-directory"; Enter $p
Start-Sleep 2; Shot $p "04-finish"; Enter $p
$p.WaitForExit(15000) | Out-Null
$u = Start-Process "$env:LOCALAPPDATA\Programs\GDM-Spike\uninstall.exe" -PassThru
Start-Sleep 2
$u = Get-Process -Name "Un" | Sort-Object StartTime | Select-Object -Last 1   # re-launched as %TEMP%\~nsuN.tmp\Un.exe
Shot $u "05-uninstall-confirm"; Enter $u   # language comes from HKCU (MUI_LANGDLL_REGISTRY_*), no dialog
Start-Sleep 2; Shot $u "06-uninstall-done"; Enter $u
