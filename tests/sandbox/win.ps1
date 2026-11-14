# SPDX-License-Identifier: RPL-1.5
# Desktop helpers for tests/sandbox/qa.mjs (inside Windows Sandbox): real-screen capture, real OS mouse input,
# window facts. Physical pixels everywhere (the process is made DPI aware first).
#   shot <png>            -> "ok" | "blank" (one colour: the session is minimized/locked => inconclusive)
#   click <x> <y>         -> moves the real cursor and clicks (SendInput path, like a user)
#   fg                    -> "<process>|<title>" of the foreground window
#   win <process>         -> "x y w h" of that process's visible main window, client origin "cx cy", or "none"
#   front <process>       -> brings that process's main window to the front
#   screen                -> primary monitor size "w h"
#   area                  -> work area of the primary monitor "x y w h"
param([string]$cmd, [string]$a1, [string]$a2)
Add-Type -ReferencedAssemblies System.Drawing, System.Windows.Forms -TypeDefinition @"
using System; using System.Runtime.InteropServices; using System.Drawing; using System.Text;
public static class W {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint x, uint y, uint d, IntPtr e);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern void keybd_event(byte k, byte s, uint f, IntPtr e);
  [DllImport("user32.dll")] public static extern int GetWindowThreadProcessId(IntPtr h, out int pid);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [StructLayout(LayoutKind.Sequential)] public struct R { public int l, t, r, b; }
  [StructLayout(LayoutKind.Sequential)] public struct P { public int x, y; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out R r);
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h, int a, out R r, int n);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref P p);
  [DllImport("user32.dll")] public static extern bool SystemParametersInfo(uint a, uint b, out R r, uint c);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
  // the process's real top-level window: visible, titled, the biggest one (Process.MainWindowHandle is unreliable
  // for multi-process apps like Chromium and for GDM's hidden tray window)
  public static IntPtr Top(string name, bool iconicToo) {
    IntPtr best = IntPtr.Zero; long area = -1;
    EnumWindows((h, l) => {
      if (!IsWindowVisible(h) || (!iconicToo && IsIconic(h))) return true;
      var t = new StringBuilder(256); if (GetWindowText(h, t, 256) == 0) return true;
      int pid; GetWindowThreadProcessId(h, out pid);
      string pn; try { pn = System.Diagnostics.Process.GetProcessById(pid).ProcessName; } catch { return true; }
      if (!string.Equals(pn, name, StringComparison.OrdinalIgnoreCase)) return true;
      R r; GetWindowRect(h, out r); long a = (long)(r.r - r.l) * (r.b - r.t);
      if (a > area) { area = a; best = h; }
      return true;
    }, IntPtr.Zero);
    return best;
  }
}
"@
[W]::SetProcessDPIAware() | Out-Null
switch ($cmd) {
  "shot" {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($b.Location, [System.Drawing.Point]::Empty, $b.Size)
    $first = $bmp.GetPixel(0, 0); $same = $true
    for ($i = 1; $i -lt 400 -and $same; $i++) { $p = $bmp.GetPixel(($i * 7919) % $b.Width, ($i * 104729) % $b.Height); if ($p -ne $first) { $same = $false } }
    $bmp.Save($a1, [System.Drawing.Imaging.ImageFormat]::Png); $g.Dispose(); $bmp.Dispose()
    if ($same) { "blank" } else { "ok" }
  }
  "click" {
    [W]::SetCursorPos([int]$a1, [int]$a2) | Out-Null; Start-Sleep -Milliseconds 120
    [W]::mouse_event(2, 0, 0, 0, [IntPtr]::Zero); Start-Sleep -Milliseconds 60; [W]::mouse_event(4, 0, 0, 0, [IntPtr]::Zero)
    "ok"
  }
  "fg" {
    $h = [W]::GetForegroundWindow(); $id = 0; [W]::GetWindowThreadProcessId($h, [ref]$id) | Out-Null
    $s = New-Object System.Text.StringBuilder 512; [W]::GetWindowText($h, $s, 512) | Out-Null
    (Get-Process -Id $id -ErrorAction SilentlyContinue).ProcessName + "|" + $s.ToString()
  }
  "win" {
    $h = [W]::Top($a1, $false)
    if ($h -eq [IntPtr]::Zero) { "none"; break }
    # the visible frame (DWM extended frame bounds): GetWindowRect also counts the invisible resize borders
    $r = New-Object W+R
    if ([W]::DwmGetWindowAttribute($h, 9, [ref]$r, 16) -ne 0) { [W]::GetWindowRect($h, [ref]$r) | Out-Null }
    $c = New-Object W+P; [W]::ClientToScreen($h, [ref]$c) | Out-Null
    "$($r.l) $($r.t) $($r.r - $r.l) $($r.b - $r.t) $($c.x) $($c.y)"
  }
  "front" {
    # what a user does before clicking in the browser while GDM's window is over it: GDM goes down (minimized:
    # a capture must bring it back by itself), the browser comes up maximized. Moving the foreground from another
    # process is not always allowed by Windows; minimizing is.
    $g = [W]::Top("gdm", $false)
    if ($g -ne [IntPtr]::Zero) { [W]::ShowWindow($g, 6) | Out-Null }
    $h = [W]::Top($a1, $true)
    if ($h -eq [IntPtr]::Zero) { "none"; break }
    [W]::keybd_event(0x12, 0, 0, [IntPtr]::Zero); [W]::keybd_event(0x12, 0, 2, [IntPtr]::Zero)
    [W]::ShowWindow($h, 3) | Out-Null
    [W]::SetForegroundWindow($h) | Out-Null
    Start-Sleep -Milliseconds 400
    "ok"
  }
  "screen" {
    $b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    "$($b.Width) $($b.Height)"
  }
  "area" {
    $r = New-Object W+R; [W]::SystemParametersInfo(0x30, 0, [ref]$r, 0) | Out-Null
    "$($r.l) $($r.t) $($r.r - $r.l) $($r.b - $r.t)"
  }
}
