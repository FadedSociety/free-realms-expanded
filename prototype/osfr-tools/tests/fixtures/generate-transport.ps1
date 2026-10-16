param([string]$LabRoot = '')
$ErrorActionPreference = 'Stop'
if (!$LabRoot) { $LabRoot = Join-Path $PSScriptRoot '..\..\..\..\..\lab' }
$LabRoot = [IO.Path]::GetFullPath($LabRoot)
$assemblyPath = Join-Path $LabRoot 'working\OSFR.Server\OSFR Server\LibSOE.dll'
$assembly = [Reflection.Assembly]::LoadFrom($assemblyPath)
[void][Reflection.Assembly]::LoadFrom((Join-Path (Split-Path $assemblyPath) 'DotNetZip.dll'))
$flags = [Reflection.BindingFlags]'Public,NonPublic,Instance'
function Empty-Instance([string]$name) {
    [Runtime.Serialization.FormatterServices]::GetUninitializedObject($assembly.GetType($name, $true))
}
function Hex([byte[]]$bytes) { [BitConverter]::ToString($bytes).Replace('-', '').ToLowerInvariant() }
$protocol = Empty-Instance 'SOE.Core.SOEProtocol'
$server = Empty-Instance 'SOE.Core.SOEServer'
$server.GetType().GetField('Protocol', $flags).SetValue($server, $protocol)
$client = Empty-Instance 'SOE.Core.SOEClient'
$client.Server = $server
$rows = [Collections.Generic.List[string]]::new()
$rows.Add("name`tseed`tcrc_length`tcompression`tpayload_hex`tpacket_hex`tchecksum_hex")
$cases = @(
    @{ Name='ack'; OpCode=21; Payload=[byte[]](0x12,0x34); Compression=$true },
    @{ Name='data'; OpCode=9; Payload=[byte[]](0,0,1,0,0x44,0x55,0x66); Compression=$true },
    @{ Name='first-fragment'; OpCode=13; Payload=[byte[]](0,1,0,0,0,6,0x61,0x62); Compression=$true },
    @{ Name='no-compression'; OpCode=9; Payload=[byte[]](0,0,0,0xff,0x17); Compression=$false },
    @{ Name='threshold-100'; OpCode=9; Payload=[byte[]](@(0,0) + (0..97)); Compression=$true },
    @{ Name='compressed-101'; OpCode=9; Payload=[byte[]](@(0,0) + (@(0x61)*99)); Compression=$true },
    @{ Name='compressed-binary'; OpCode=9; Payload=[byte[]](@(0xff,0xff) + (0..255)); Compression=$true }
)
foreach ($seed in [uint32[]](0,1884358976,[uint32]::MaxValue)) {
    foreach ($crcLength in 0..4) {
        $client.StartSession([uint32]$crcLength, [uint32]0x12345678, [uint32]512)
        $client.GetType().GetField('CRCSeed', $flags).SetValue($client, $seed)
        foreach ($case in $cases) {
            $client.SetCompressable($case.Compression)
            $writer = [Activator]::CreateInstance($assembly.GetType('SOE.Interfaces.SOEWriter'), [object[]]([uint16]$case.OpCode, $false))
            $writer.AddBytes($case.Payload)
            $packet = $writer.GetFinalSOEPacket($client, $true, $true).GetRaw()
            $withoutCrc = [byte[]]::new($packet.Length - $crcLength)
            [Array]::Copy($packet, $withoutCrc, $withoutCrc.Length)
            $checksum = $protocol.GetCRC32Checksum($seed, $withoutCrc)
            $rows.Add(($case.Name, $seed, $crcLength, ([int]$case.Compression), (Hex $case.Payload), (Hex $packet), $checksum.ToString('x8') -join "`t"))
        }
    }
}
[IO.File]::WriteAllLines((Join-Path $PSScriptRoot 'transport-vectors.tsv'), $rows)
$metadata = [ordered]@{
    source = $assemblyPath
    source_sha256 = (Get-FileHash -LiteralPath $assemblyPath -Algorithm SHA256).Hash.ToLowerInvariant()
    methods = @('SOEWriter.GetFinalSOEPacket', 'SOEProtocol.GetCRC32Checksum')
    vector_count = $rows.Count - 1
    constructor_execution = $false
    network_execution = $false
    note = 'Synthetic payloads and seeds. Objects allocated without constructors; only serializer, compression and checksum methods execute.'
}
$metadata | ConvertTo-Json -Depth 3 | Set-Content -LiteralPath (Join-Path $PSScriptRoot 'transport-provenance.json') -Encoding utf8
Write-Output ("Generated " + ($rows.Count - 1) + ' original-serializer transport vectors')
