# Transport vector provenance

`transport-vectors.tsv` contains 105 synthetic frames produced by the **compiled user-supplied** `LibSOE.dll`, not the Rust implementation. `generate-transport.ps1` reproduces them from the preserved working copy and records its SHA-256 in `transport-provenance.json`.

The generator creates `SOEServer`, `SOEProtocol` and `SOEClient` objects without invoking constructors. It assigns the protocol reference and negotiated fields, then calls the original `SOEWriter.GetFinalSOEPacket` and `SOEProtocol.GetCRC32Checksum` methods. No sockets, gameplay or server lifecycle methods execute.

The matrix covers seeds 0, 1884358976 and 4294967295; CRC trailer lengths 0–4; acknowledgement, data and first-fragment frames; compression disabled; and compression thresholds at payload lengths 100 and 101. Payloads contain only chosen synthetic bytes.

Rust tests compare every checksum and decode every original frame. Uncompressed frame encoding must match exactly. Zlib encoding is compared by recovered bytes, because conforming compressor implementations can emit different streams.

The ignored test `compiled_legacy_decoder_accepts_rust_compression` writes five temporary synthetic Rust frames, then invokes `tests/check-legacy-decode.ps1`. The compiled original checksum and private decompression methods must accept them, including a 200,000-byte expanded payload. Run it explicitly from the project in PowerShell with `& .\scripts\cargo.ps1 -CargoArgs @('test','--manifest-path','prototype/osfr-tools/Cargo.toml','--test','soe_transport','compiled_legacy_decoder_accepts_rust_compression','--','--ignored','--nocapture')`. This test needs the user's local assembly and Windows PowerShell; ordinary tests only need the checked-in synthetic vectors.

This is evidence for this supplied emulator's wire serializer. Actual client interoperability remains a separate acceptance test. Opcode 17's interpretation as a selective acknowledgement also follows Sanctuary's newer `UdpReliableChannel.ReliablePacket`; legacy LibSOE calls the same opcode "out of order". No Sanctuary source is copied into this implementation.

## Explicit local capture replay

Run `scripts/verify-soe-capture.ps1 -CaptureDirectory <capture-folder-under-lab>` to exercise the Rust codec and receiver against a separately recorded legacy-server capture. The runner verifies every manifest-referenced file's length and SHA-256, creates an index containing paths and metadata in the lab, and invokes the ignored `tests/soe_capture.rs` replay. Rust decodes all received post-session frames, rejects a copy of each frame with a modified CRC, reassembles reliable messages, and compares each message byte-for-byte with the verified original files. Reports remain under `lab/manifests`; no captured payloads or game keys are embedded in the repository.

The first recorded capture (`legacy-gateway-20261008T014620082859Z`) passed: 1,305 input-file hashes; 1,253 decoded frames including 1,237 compressed frames; 46 reassembled messages totaling 627,566 bytes; and 1,253 CRC corruption rejections. The capture was generated through synthetic local requests to the supplied server. This proves compatibility with those actual server outputs, not original-game-client login.
