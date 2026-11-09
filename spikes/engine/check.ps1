# SPDX-License-Identifier: RPL-1.5
# Runs one G3 spike check (exit 0 = pass). Build output stays outside OneDrive.
#   .\check.ps1 s2|s3|s4|s5|s6|s7|s8 [args]     e.g. .\check.ps1 s5 1000
param([Parameter(Mandatory)][string]$Spike, [Parameter(ValueFromRemainingArguments)][string[]]$Rest)
$env:CARGO_TARGET_DIR = "$env:LOCALAPPDATA\gdm-spike-target"
Push-Location $PSScriptRoot
try {
    cargo build --release --quiet --bin $Spike
    if ($LASTEXITCODE) { exit $LASTEXITCODE }
    & "$env:CARGO_TARGET_DIR\release\$Spike.exe" @Rest
    exit $LASTEXITCODE
} finally { Pop-Location }
