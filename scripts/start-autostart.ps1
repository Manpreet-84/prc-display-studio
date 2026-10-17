$ErrorActionPreference = 'Stop'
$project = Split-Path $PSScriptRoot -Parent
$binary = Join-Path $project 'target/release/prc-lcd.exe'
if (-not (Test-Path -LiteralPath $binary)) { throw 'Build with cargo build --release first.' }
$env:PRC_THEME_DIR = Join-Path $project 'themes'
Start-Process -FilePath $binary -ArgumentList '--live','custom','--loop' -WorkingDirectory $project -WindowStyle Hidden
