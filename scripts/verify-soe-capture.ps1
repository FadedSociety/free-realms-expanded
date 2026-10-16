param([Parameter(Mandatory=$true)][string]$CaptureDirectory)
$ErrorActionPreference = 'Stop'
$captureRoot = [IO.Path]::GetFullPath($CaptureDirectory).TrimEnd('\', '/')
$capturePrefix = $captureRoot + [IO.Path]::DirectorySeparatorChar
$manifestPath = Join-Path $captureRoot 'manifest.json'
$manifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($manifest.status -ne 'passed' -or $manifest.crc_length -lt 1 -or $manifest.crc_length -gt 4) {
    throw 'Capture must have passed and contain a negotiated CRC length between 1 and 4'
}
$projectRoot = [IO.Path]::GetFullPath((Join-Path $PSScriptRoot '..'))
$labRoot = [IO.Path]::GetFullPath((Join-Path $projectRoot '..\lab'))
$reportRoot = Join-Path $labRoot ('manifests\soe-rust-replay-' + [DateTime]::UtcNow.ToString('yyyyMMddTHHmmssfffffffZ'))
[void](New-Item -ItemType Directory -Path $reportRoot)
$rows = [Collections.Generic.List[string]]::new()
$rows.Add("seed`t" + $manifest.crc_seed)
$rows.Add("crc_length`t" + $manifest.crc_length)
$rows.Add("expected_messages`t" + $manifest.messages.Count)
$rows.Add("expected_application_bytes`t" + ($manifest.messages | Measure-Object length -Sum).Sum)
$hashChecks = 0
function Verify-Input($entry) {
    $path = [IO.Path]::GetFullPath((Join-Path $captureRoot $entry.path))
    if (!$path.StartsWith($capturePrefix, [StringComparison]::OrdinalIgnoreCase) -or $path.Contains("`t") -or $path.Contains("`n")) {
        throw 'Capture index contains an invalid path'
    }
    $file = Get-Item -LiteralPath $path
    if ($file.Length -ne $entry.length) { throw ('Length mismatch: ' + $entry.path) }
    $hash = (Get-FileHash -LiteralPath $path -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($hash -ne $entry.sha256) { throw ('SHA-256 mismatch: ' + $entry.path) }
    return $path
}
foreach ($frame in $manifest.frames) {
    $path = Verify-Input $frame
    $hashChecks++
    if ($frame.direction -eq 'rx' -and $frame.phase -ne 'session') {
        $rows.Add((@('frame', $path, $frame.opcode, $frame.expanded_body_length) -join "`t"))
    }
}
foreach ($message in $manifest.messages) {
    $path = Verify-Input $message
    $hashChecks++
    $rows.Add((@('message', $path, $message.length) -join "`t"))
}
$indexPath = Join-Path $reportRoot 'inputs.tsv'
$rustReportPath = Join-Path $reportRoot 'rust-replay.json'
$rows | Set-Content -LiteralPath $indexPath -Encoding utf8
$previousIndex = $env:OSFR_CAPTURE_INDEX
$previousReport = $env:OSFR_CAPTURE_REPORT
try {
    $env:OSFR_CAPTURE_INDEX = $indexPath
    $env:OSFR_CAPTURE_REPORT = $rustReportPath
    $cargoOutput = & (Join-Path $PSScriptRoot 'cargo.ps1') -CargoArgs @(
        'test','--manifest-path',(Join-Path $projectRoot 'prototype\osfr-tools\Cargo.toml'),
        '--test','soe_capture','replay_local_legacy_capture','--','--ignored','--exact','--nocapture'
    ) 2>&1
    $cargoExit = $LASTEXITCODE
    $cargoOutput | Set-Content -LiteralPath (Join-Path $reportRoot 'cargo.log') -Encoding utf8
    if ($cargoExit -ne 0) {
        $cargoOutput | Write-Output
        throw ('Rust capture replay failed; evidence retained at ' + $reportRoot)
    }
    $report = Get-Content -LiteralPath $rustReportPath -Raw | ConvertFrom-Json
    $summary = [ordered]@{
        status = 'passed'
        capture_directory = $captureRoot
        manifest_sha256 = (Get-FileHash -LiteralPath $manifestPath -Algorithm SHA256).Hash.ToLowerInvariant()
        source_server_sha256 = $manifest.server_sha256
        source_game_client_launched = $manifest.game_client_launched
        source_client_requests = $manifest.client_requests
        input_files_sha256_verified = $hashChecks
        rust = $report
        report_directory = $reportRoot
    }
    $summary | ConvertTo-Json -Depth 5 | Set-Content -LiteralPath (Join-Path $reportRoot 'summary.json') -Encoding utf8
    $summary | ConvertTo-Json -Depth 5 | Write-Output
} finally {
    $env:OSFR_CAPTURE_INDEX = $previousIndex
    $env:OSFR_CAPTURE_REPORT = $previousReport
}
