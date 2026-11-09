# SPIKE ONLY (G2/S9): capture the Khmer installer at other display scales, then ALWAYS restore.
# SPI_SETLOGICALDPIOVERRIDE (0x9F) is undocumented: the value is a step relative to the recommended
# scale (Settings > Display). Measured here: 0 = 100% (recommended), 1 = 125%, 3 = 175%.
param([string]$Out, [int[]]$Steps = @(-1, 2, 4), [int]$Restore = 1)  # this machine: recommended = 100% (step 0); the owner uses 125% (step 1)
Add-Type -Name P -Namespace W -MemberDefinition @"
[DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, int b, IntPtr c, uint d);
[DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
[DllImport("user32.dll")] public static extern int GetDpiForSystem();
"@
function SetStep($s) { [W.P]::SystemParametersInfo(0x9F, $s, [IntPtr]::Zero, 1) | Out-Null; Start-Sleep 4 }
try {
  foreach ($s in $Steps) {
    SetStep $s
    $probe = powershell -NoProfile -Command "Add-Type -Name Q -Namespace W -MemberDefinition '[DllImport(\""user32.dll\"")] public static extern bool SetProcessDPIAware(); [DllImport(\""user32.dll\"")] public static extern int GetDpiForSystem();'; [W.Q]::SetProcessDPIAware()|Out-Null; [W.Q]::GetDpiForSystem()"
    $pct = [int]([int]$probe * 100 / 96)
    "== step $s -> new-process system dpi $probe ($pct%)"
    & powershell -NoProfile -ExecutionPolicy Bypass -File "$PSScriptRoot\screens.ps1" -Out "$Out\dpi-$pct" 2>&1 | Where-Object { $_ -match '^\s+\d\d-' }
  }
} finally {
  SetStep $Restore
  "== restored step $Restore"
}
