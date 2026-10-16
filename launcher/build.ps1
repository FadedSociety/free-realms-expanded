param(
    [ValidateSet('build', 'test')][string]$Action = 'build',
    [string]$LlvmMingw = $env:LLVM_MINGW_ROOT,
    [switch]$Release
)
$ErrorActionPreference = 'Stop'
$lab = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..\..\lab'))
if (!$LlvmMingw) {
    $LlvmMingw = Join-Path $lab 'tools\llvm-mingw-20261006-ucrt-x86_64'
}
$linker = Join-Path $LlvmMingw 'bin\x86_64-w64-mingw32-clang.exe'
if (!(Test-Path -LiteralPath $linker -PathType Leaf)) {
    throw 'Set LLVM_MINGW_ROOT or -LlvmMingw to an unpacked llvm-mingw UCRT toolchain.'
}
# Do not add llvm-mingw/bin to PATH: host build scripts must keep their host compiler.
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_LINKER = [IO.Path]::GetFullPath($linker)
# Set this for every invocation; the failing old artifact had no target rustflags.
$env:CARGO_TARGET_X86_64_PC_WINDOWS_GNULLVM_RUSTFLAGS = '-C target-feature=+crt-static'
if (!$env:CARGO_TARGET_DIR) {
    $env:CARGO_TARGET_DIR = Join-Path $lab 'build\w86b\target'
}
$cargoArgs = @($Action, '--offline', '--locked', '--target', 'x86_64-pc-windows-gnullvm',
    '--manifest-path', (Join-Path $PSScriptRoot 'Cargo.toml'))
if ($Release) { $cargoArgs += '--release' }
& (Join-Path $PSScriptRoot '..\scripts\cargo.ps1') @cargoArgs
exit $LASTEXITCODE
