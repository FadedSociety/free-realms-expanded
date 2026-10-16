# SOE transport notes

Evidence source: the supplied `LibSOE.dll`, recovered under `lab/decompiled/server/LibSOE`. These are observations from this emulator version, not a complete protocol specification or proof of original-client behavior.

## Initial session

The implemented Rust probe expects the following big-endian fields: opcode u16 (`1`), CRC length u32, session ID u32, UDP buffer size u32, then the null-terminated `CGAPI_527` string. Its 21-byte response has opcode `2`, the echoed session ID, CRC seed `1884358976`, one-byte CRC length, compression flag `1`, encryption flag `0`, echoed buffer size, and trailing u32 `3`.

The supplied SOEProtocol.HandleSessionRequest and SOEClient.StartSession are the evidence. Synthetic request and response fixtures are documented in `prototype/osfr-tools/tests/fixtures/README.md`. The actual supplied client has now been captured: it negotiates a three-byte CRC trailer and successfully enters the emulator world. See [[Client Baseline]]. The earlier synthetic oracle requested two bytes; fixtures must preserve that distinction.

A live check of a temporary copy of the supplied OSFRServer.exe returned the same 21 response bytes for the synthetic request. See `lab/manifests/legacy-handshake-probe.json`; the test server was stopped afterward.

## Subsequent framing to verify

| Feature | Observed legacy implementation | Verification still needed |
| --- | --- | --- |
| Transport byte order | SOEReader/Writer network methods are big-endian; Host variants are little-endian | Check each application field independently |
| Data opcode | `9`, followed by u16 sequence | Real-client trace and loss/reordering behavior |
| Fragment opcode | `13`; initial fragment includes total message length | Fragment boundaries, retransmission, bounds, and rollover |
| Acknowledgement | Opcode `21`, sequence u16 | Cumulative vs individual semantics from traces/newer implementation |
| Out-of-order notice | Opcode `17`, sequence u16 | Client response and resend behavior |
| Ping | Opcode `6` | Session exceptions to framing and checksums |
| Compression | Flag after transport opcode; `1` invokes zlib | Flag coverage and maximum expanded size |
| CRC | Seed contributes to CRC-32; append the negotiated number of low-order checksum bytes in big-endian order | Golden vectors and corrupted-frame rejection |
| Application messages | Gateway definitions and reader Host methods supply separate conventions | Actual login and world-initialization sequence |

## Legacy behavior requiring caution during porting

Static inspection of SOEProtocol.HandlePacket shows its CRC comparison loop starts at `4 - CRCLength` and runs while the index is less than `CRCLength`. With a two-byte CRC it performs no comparisons. Rust should verify the negotiated trailer against independent vectors; copying this loop would preserve the apparent defect. This finding has not been exercised against a running legacy server.

SOEDataChannel.ReceiveMessage assigns LastReceivedSequenceNumber from the incoming sequence minus one before its sequence check. That check therefore does not establish packet ordering against prior state. Fragment paths also warrant independent tests before reuse. These observations explain why the next Rust step needs packet-loss, duplication, reordering, truncation, and fragment-bound tests, not only successful handshake fixtures.

Use Sanctuary's newer UDP implementation as another evidence source while retaining its provenance. Reconcile differences with actual client behavior rather than treating either emulator as authoritative by itself.


## Verified Rust receive slice

`prototype/osfr-tools/src/soe.rs` now supplies the bounded codec and channel-zero reliable receiver. It validates CRCs and zlib framing, bounds expanded messages, buffers future sequences within a bounded window, suppresses duplicates, and assembles fragments through rollover. Future data is length-checked before selective acknowledgement; a queued packet later found invalid poisons that receiver. Direct errors leave receive state unchanged and must not be acknowledged. No socket server or reliable sender exists yet.

Verification: 105 compiled serializer vectors, corrupted-frame rejection, strict expansion/trailing-data checks, 131,104 ordered deliveries through repeated sequence rollover, and an explicit compiled .NET decoder check. Rust also replayed 1,253 actual supplied-server frames from synthetic login/ready requests and matched all 46 reassembled messages/627,566 bytes. Provenance and reproduction: `tests/fixtures/TRANSPORT.md`; [[Legacy Gateway Oracle]].

The actual-client baseline additionally demonstrates multi-message bundles beginning `00 19`, a three-byte CRC, and post-world movement/camera traffic. The successful input-free capture has contiguous server sequences 0-1258. Its single legacy out-of-order notice occurred without a gap in the captured client sequence; investigate the recovered continuation-fragment comparison defect instead of treating that notice as proof of loss. The baseline note distinguishes decoded observations from unimplemented Rust behavior. Reliable retransmission/control handling, application bundles, encryption, and persistence still require implementation and client checks.
