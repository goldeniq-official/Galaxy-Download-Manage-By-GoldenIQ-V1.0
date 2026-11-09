# SPDX-License-Identifier: RPL-1.5
# Benchmark harness for plan.md section ង: GDM spike (bench.exe) vs IDM, paired runs, seeded AB/BA order.
# Usage: .\bench.ps1 -UrlFile urls.txt -Out raw.csv [-Pairs 5] [-Seed 20261109] [-Fixed 0|8] [-Warmup 1] [-Root dir]
# urls.txt lines: id|url|bytes|sha256   (sha256 may be empty: then the warm-up pair pins it, both tools must agree)
param([string]$UrlFile, [string]$Out, [int]$Pairs = 5, [int]$Seed = 20261109, [int]$Fixed = 0, [int]$Warmup = 1, [string]$Config = "a", [string]$Root = "$env:USERPROFILE\gdm-bench", [int]$TimeoutS = 600)
$ErrorActionPreference = 'Stop'
$bench = "$env:LOCALAPPDATA\gdm-spike-target\release\bench.exe"
$idm = "C:\Program Files (x86)\Internet Download Manager\IDMan.exe"
$root = "$Root\run-$Seed-$Config"
New-Item -ItemType Directory -Force $root | Out-Null
$rng = New-Object System.Random $Seed

function Restart-Idm {
    Get-Process IDMan -ErrorAction SilentlyContinue | Stop-Process -Force
    Start-Sleep 1
    Start-Process $idm | Out-Null
    for ($i = 0; $i -lt 50 -and -not (Get-Process IDMan -ErrorAction SilentlyContinue); $i++) { Start-Sleep -Milliseconds 200 }
    Start-Sleep 3   # IDM start-up is not part of the download time
}

function Measure-Run($tool, $u, $dir, $name) {
    ipconfig /flushdns | Out-Null
    $tp = Join-Path $dir "cpu-$tool.csv"
    $typeperf = Start-Process typeperf -ArgumentList "`"\Processor(_Total)\% Processor Time`" -si 1 -o `"$tp`" -y" -PassThru -WindowStyle Hidden
    $peakConn = 0; $peakPriv = 0; $cpu0 = $null
    $sw = [Diagnostics.Stopwatch]::StartNew()
    $line = $null
    if ($tool -eq 'GDM') {
        $al = @("`"$($u.url)`"", "`"$dir`"", $name); if ($Fixed -gt 0) { $al += "$Fixed" }
        $p = Start-Process $bench -ArgumentList $al -PassThru -NoNewWindow -RedirectStandardOutput (Join-Path $dir "gdm.out")
        $procId = $p.Id
        while (-not $p.HasExited) {
            if ($sw.Elapsed.TotalSeconds -gt $TimeoutS) { Stop-Process -Id $procId -Force -ErrorAction SilentlyContinue; break }  # stalled run = timeout, recorded
            try { $q = Get-Process -Id $procId -ErrorAction Stop; $peakPriv = [math]::Max($peakPriv, $q.PrivateMemorySize64) } catch {}
            $c = @(Get-NetTCPConnection -OwningProcess $procId -State Established -ErrorAction SilentlyContinue).Count; $peakConn = [math]::Max($peakConn, $c)
            Start-Sleep -Milliseconds 250
        }
        $done = $sw.Elapsed.TotalSeconds
        $cpuSec = $p.TotalProcessorTime.TotalSeconds
        $line = Get-Content (Join-Path $dir "gdm.out") -Raw -ErrorAction SilentlyContinue
        if ($line -and $line.Trim()) { $parts = $line.Trim().Split(','); $transfer = [double]$parts[1]; $segPeak = [int]$parts[3] } else { $transfer = ''; $segPeak = '' }  # timeout / failure
    } else {
        $q = Get-Process IDMan; $procId = $q.Id; $cpu0 = $q.TotalProcessorTime.TotalSeconds
        Start-Process $idm -ArgumentList "/d `"$($u.url)`" /p `"$dir`" /f `"$name`" /n" | Out-Null
        $f = Join-Path $dir $name
        while ($sw.Elapsed.TotalSeconds -lt $TimeoutS) {  # a stalled IDM run is recorded as timeout (sha missing)
            try { $q = Get-Process -Id $procId -ErrorAction Stop; $peakPriv = [math]::Max($peakPriv, $q.PrivateMemorySize64) } catch {}
            $c = @(Get-NetTCPConnection -OwningProcess $procId -State Established -ErrorAction SilentlyContinue | Where-Object { $_.RemoteAddress -ne '127.0.0.1' -or $u.url -like 'http://127.0.0.1*' }).Count; $peakConn = [math]::Max($peakConn, $c)
            $it = Get-Item $f -ErrorAction SilentlyContinue
            if ($it -and $it.Length -eq [long]$u.bytes) { try { $s = [IO.File]::Open($f, 'Open', 'Read', 'None'); $s.Close(); break } catch {} }
            Start-Sleep -Milliseconds 100
        }
        $done = $sw.Elapsed.TotalSeconds
        $cpuSec = (Get-Process -Id $procId).TotalProcessorTime.TotalSeconds - $cpu0
        $transfer = ''; $segPeak = ''
    }
    Stop-Process -Id $typeperf.Id -Force -ErrorAction SilentlyContinue
    Start-Sleep -Milliseconds 300
    $cpuRows = @(Import-Csv $tp -ErrorAction SilentlyContinue | ForEach-Object { ($_.PSObject.Properties | Select-Object -Last 1).Value -as [double] } | Where-Object { $null -ne $_ })  # typeperf leaves a blank last sample when stopped
    $cpuAvg = if ($cpuRows.Count) { ($cpuRows | Measure-Object -Average).Average } else { -1 }
    $cpuMax = if ($cpuRows.Count) { ($cpuRows | Measure-Object -Maximum).Maximum } else { -1 }
    $file = Join-Path $dir $name
    $sha = if (Test-Path $file) { (Get-FileHash $file).Hash.ToLower() } else { 'missing' }
    [pscustomobject]@{ tool = $tool; done_s = [math]::Round($done, 3); transfer_s = $transfer; MBps = [math]::Round([long]$u.bytes / 1e6 / $done, 2); tcp_conns_peak = $peakConn; gdm_segments_peak = $segPeak; cpu_machine_avg = [math]::Round($cpuAvg, 1); cpu_machine_max = [math]::Round($cpuMax, 1); proc_cpu_s = [math]::Round($cpuSec, 2); priv_MB_peak = [math]::Round($peakPriv / 1MB, 1); sha256 = $sha }
}

$urls = Get-Content $UrlFile | Where-Object { $_ -and -not $_.StartsWith('#') } | ForEach-Object { $a = $_.Split('|'); [pscustomobject]@{ id = $a[0]; url = $a[1]; bytes = $a[2]; sha = $a[3] } }
$rows = @()
foreach ($u in $urls) {
    # AB/BA order, seeded and balanced as far as an odd count allows
    $orders = @(1..$Pairs | ForEach-Object { if ($_ % 2) { 'AB' } else { 'BA' } }) | Sort-Object { $rng.Next() }
    for ($i = 1 - $Warmup; $i -le $Pairs; $i++) {
        $order = if ($i -le 0) { 'AB' } else { $orders[$i - 1] }
        $tools = if ($order -eq 'AB') { 'GDM', 'IDM' } else { 'IDM', 'GDM' }
        foreach ($t in $tools) {
            $dir = Join-Path $root "$($u.id)-$i-$t"; New-Item -ItemType Directory -Force $dir | Out-Null
            if ($t -eq 'IDM') { Restart-Idm }
            $r = Measure-Run $t $u $dir "$($u.id).bin"
            if ($i -le 0 -and -not $u.sha) { if ($t -eq 'GDM') { $u.sha = $r.sha256 } elseif ($u.sha -ne $r.sha256) { Write-Warning "$($u.id): GDM and IDM warm-up hashes differ" } }
            $r | Add-Member url_id $u.id; $r | Add-Member pair $i; $r | Add-Member order $order; $r | Add-Member config $Config; $r | Add-Member sha_ok ($r.sha256 -eq $u.sha)
            $rows += $r
            Write-Host ("{0} pair {1} {2} {3}: {4}s {5} MB/s conns {6} sha_ok {7}" -f $u.id, $i, $order, $t, $r.done_s, $r.MBps, $r.tcp_conns_peak, $r.sha_ok)
            Get-ChildItem $dir -Filter "*.bin" | ForEach-Object { [IO.File]::Delete($_.FullName) }   # free disk between runs
            $rows | Export-Csv $Out -NoTypeInformation   # keep raw data even if a later run fails
        }
    }
    Write-Host "$($u.id) pinned sha256 $($u.sha)"
}
