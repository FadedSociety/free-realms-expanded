param([Parameter(ValueFromRemainingArguments=$true)][string[]]$CargoArgs)
$ErrorActionPreference = 'Stop'
$toolRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\lab\tools'))
$env:CARGO_HOME = Join-Path $toolRoot 'cargo'
$env:RUSTUP_HOME = Join-Path $toolRoot 'rustup'
$env:PATH = (Join-Path $env:CARGO_HOME 'bin') + ';' + $env:PATH
& (Join-Path $env:CARGO_HOME 'bin\cargo.exe') @CargoArgs
exit $LASTEXITCODE
