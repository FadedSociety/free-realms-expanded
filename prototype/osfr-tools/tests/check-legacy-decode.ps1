param(
    [Parameter(Mandatory=$true)][string]$Vectors,
    [string]$LabRoot = ''
)
$ErrorActionPreference = 'Stop'
if (!$LabRoot) { $LabRoot = Join-Path $PSScriptRoot '..\..\..\..\lab' }
$LabRoot = [IO.Path]::GetFullPath($LabRoot)
$assemblyPath = Join-Path $LabRoot 'working\OSFR.Server\OSFR Server\LibSOE.dll'
$assembly = [Reflection.Assembly]::LoadFrom($assemblyPath)
[void][Reflection.Assembly]::LoadFrom((Join-Path (Split-Path $assemblyPath) 'DotNetZip.dll'))
$flags = [Reflection.BindingFlags]'Public,NonPublic,Instance'
function Empty-Instance([string]$name) {
    [Runtime.Serialization.FormatterServices]::GetUninitializedObject($assembly.GetType($name, $true))
}
function Bytes([string]$hex) {
    $bytes = [byte[]]::new($hex.Length / 2)
    for ($i = 0; $i -lt $bytes.Length; $i++) { $bytes[$i] = [Convert]::ToByte($hex.Substring($i * 2, 2), 16) }
    return ,$bytes
}
function Hex([byte[]]$bytes) { [BitConverter]::ToString($bytes).Replace('-', '').ToLowerInvariant() }
$protocol = Empty-Instance 'SOE.Core.SOEProtocol'
$client = Empty-Instance 'SOE.Core.SOEClient'
$client.StartSession(2, [uint32]0x12345678, 512)
$decompress = $protocol.GetType().GetMethod('DecompressPacket', $flags)
$count = 0
foreach ($row in Import-Csv -LiteralPath $Vectors -Delimiter "`t") {
    $wire = Bytes $row.packet_hex
    $withoutCrc = [byte[]]::new($wire.Length - 2)
    [Array]::Copy($wire, $withoutCrc, $withoutCrc.Length)
    $crc = $protocol.GetCRC32Checksum([uint32]1884358976, $withoutCrc)
    $wantTrailer = [uint32]$wire[$wire.Length - 2] * 256 + $wire[$wire.Length - 1]
    if (($crc -band 65535) -ne $wantTrailer) { throw ('CRC rejected for ' + $row.name) }
    $cooked = $decompress.Invoke($protocol, [object[]]($client, $withoutCrc))
    if ((Hex $cooked) -ne ('0009' + $row.payload_hex)) { throw ('Original decoder mismatch for ' + $row.name) }
    $count++
}
if ($count -ne 5) { throw 'Expected five independent decoder comparisons' }
Write-Output ("Original compiled decoder accepted " + $count + ' Rust frames')
