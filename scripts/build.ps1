$ErrorActionPreference = 'Stop'
Set-Location (Split-Path $PSScriptRoot -Parent)
if (Test-Path '.tools/cargo/bin/cargo.exe') {
  $env:CARGO_HOME = Join-Path $PWD '.tools/cargo'
  $env:RUSTUP_HOME = Join-Path $PWD '.tools/rustup'
  $env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
}
npm.cmd run tauri build
exit $LASTEXITCODE
