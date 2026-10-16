# Session handshake fixture provenance

These two short wire-format test vectors were generated locally through `SOE.Interfaces.SOEWriter` from the user-supplied `LibSOE.dll`, using the field sequence recovered in `SOEProtocol.HandleSessionRequest`.

- Protocol: `CGAPI_527`, null terminated.
- Session ID: `0x12345678` (chosen test value).
- CRC length: 2; buffer size: 512.
- Response CRC seed: 1884358976, observed in `SOEClient.StartSession`.
- Compression advertised: true; encryption: false; final response integer: 3.

No game account or live traffic was used. This is a serializer comparison, not evidence that the Rust probe can complete a real game login.

A subsequent local check started a temporary copy of the supplied OSFRServer.exe and sent this synthetic request over UDP. Its response exactly matched `session-response.bin`; the process was stopped afterward. The result is recorded in `lab/manifests/legacy-handshake-probe.json`. No original game client was launched.
