# Supplied-client baseline

The supplied Free Realms client now enters and renders the local walking emulator's world. A separate, user-approved UI check also verified forward movement: screenshots show the scene and the server received changing positions. This establishes a working original-client reference for the Rust reconstruction. It does not establish restored quests, activities, persistence or a Rust game server.

## Two independently recorded runs

| Run directory under `lab/runs` | Result |
| --- | --- |
| `client-baseline-20261008T020205Z` | Bounded 90-second, no-input baseline. Actual client session, login, world initialization and 1,382 UDP datagrams captured. A news modal remained visible. |
| `client-baseline-20261008T020419Z` | Bounded 120-second baseline followed by explicitly approved UI input: close the news modal and hold W for 1,200 ms. World rendering and changing server positions verified. |

Both run manifests record the client and server processes stopped. Original archives and working inputs remain preserved; these runs use copied client/server directories, backed-up configuration, a loopback UDP relay, and a loopback asset-metadata HTTP service. The relay uses an 8 MiB receive buffer and buffers the capture before disk output. Earlier attempts that lost fragment bursts are not used as evidence of a complete protocol sequence.

The working startup configuration uses the official signed x86 legacy D3DX runtime in the copied client and a matching CRC/size response for the supplied `Assets_manifest.txt`: CRC-32 **123886846**, length **9,479,368**, SHA-256 `935bc0d9f8097df9909dc4d339770deb2e7596e3c61eb58876913431657778c8`. Each successful run records one HTTP 200 request for `/manifest.crc`. See [[Client Runtime Dependencies]] for community prerequisite and archived manifest-fix sources. No replacement asset set was inferred from matching size alone.

## Real-client transport and application evidence

Detailed packet analysis applies to the completed **020205Z** run. Its evidence is under:

`%USERPROFILE%\Documents\Projects\CartoonMMO\lab\runs\client-baseline-20261008T020205Z\analysis\packet-analysis.json`

Reconstructed application messages and bundled children are beside that JSON, inside the lab. They contain game-derived data and are not copied into the project repository. Analysis records filenames, lengths, counts and hashes, without putting login credentials or payload literals in this note.

The actual client negotiated `CGAPI_527`, a 512-byte buffer, **three CRC bytes**, compression enabled and transport encryption disabled. The server seed is 1884358976. This differs from the two-byte CRC used by the earlier synthetic handshake fixture; the real negotiated length must drive decoding.

All **1,380 post-session datagrams** passed independent seeded CRC verification. Compressed bodies were decoded using Python zlib with bounds and stream-termination checks, independently of the supplied server's receive-CRC implementation. The two remaining datagrams are the unchecksummed session request/reply. No opcode-6 ping occurred in this run; a bare two-byte legacy server ping is a known special case and must not be reported as a failed checksummed data frame in another trace.

| Transport direction | Opcode | Count |
| --- | ---: | ---: |
| Client → server | 1, session request | 1 |
| Server → client | 2, session reply | 1 |
| Client → server | 9, reliable data | 43 |
| Client → server | 13, reliable fragment | 2 |
| Client → server | 21, acknowledgement | 32 |
| Server → client | 9, reliable data | 48 |
| Server → client | 13, reliable fragment | 1,211 |
| Server → client | 21, acknowledgement | 43 |
| Server → client | 17, out-of-order notice | 1 |

Server reliable data covers sequences **0–1258**, with **zero gaps**, zero incomplete fragment groups, and 54 complete application messages. Six server messages were fragmented. Client data covers sequences **0–44**, also with zero capture gaps: 44 reliable messages expand to **57 application messages** after six `00 19` message bundles are unpacked. A bundle is a transport/application-stream container, not little-endian application opcode 6400. Its length-prefixed children must be decoded before application dispatch.

Actual login is a 51-byte application message, not the earlier two-byte synthetic request: opcode 1, a length-prefixed 15-byte ticket, 64-bit character GUID, a 14-byte version string matching the recorded client build, and an empty final string. All fields exactly consume the message. Their values remain in the private capture. The legacy handler still ignores those fields; their presence in the actual client agrees with the richer layout in Sanctuary's packet definition.

Client-ready opcode 13 occurs inside a bundled tunnel at client sequence 3, 11.187 seconds into the recorded trace. Client-finished-loading opcode 10 follows at sequence 4, 11.281 seconds. The client log independently records **World Ready** and **Initialized - Client** at local `2026-10-07 19:02:18`, at `client/Logs/FreeRealms.log:319` and `:320`.

## Actual client message counts

These counts include unpacked bundles and cover the entire no-input 020205Z run. Tunnel 5 carries client packets; tunnel 6 carries client-world packets.

| Outer/tunnel opcode | Nested opcode | Count | Meaning from recovered definitions |
| --- | ---: | ---: | --- |
| 1 | — | 1 | Gateway login |
| 6 | 88 | 1 | Set locale |
| 5 | 169 | 1 | Client initialization details |
| 5 | 109 | 6 | Client log |
| 6 | 156 | 1 | Fotomat format base packet |
| 5 | 13 | 1 | Client ready |
| 6 | 102 | 8 | Lobby game definitions base packet |
| 5 | 10 | 1 | Client finished loading |
| 5 | 126 | 19 | Camera update |
| 5 | 125 | 1 | Initial position update |
| 6 | 66 | 5 | In-game purchase base packet |
| 5 | 193 | 1 | Announcement base packet |
| 5 | 52 | 7 | Game time sync |
| 5 | 105 | 4 | Client metrics |

After the client-finished-loading marker, there are 48 application messages: client logs 4, camera updates 19, initial position 1, purchase-base packets 5, announcement 1, lobby packets 7, time sync 7 and metrics 4. This packet marker is a reproducible boundary; it is not treated as an exact timestamp synchronization with every game subsystem.

The server's first 46 messages agree in structure and length with [[Legacy Gateway Oracle]]: eight login replies followed by 38 ready replies. The actual baseline then adds one area/POI update and seven time-sync replies, yielding 54. All 54 are wrapped in outer tunnel 5. The actual client accepted the older server's tunneled login reply sufficiently to reach the world; Sanctuary's top-level reply remains a version-specific protocol difference to reconcile rather than silently normalize.

## Observed legacy fragment defect

The server emits one out-of-order notice for client sequence 6 (`frames/001291.bin`), despite the independent capture showing contiguous client sequences 5 and 6 with valid CRCs and an exactly reconstructed 507-byte client-log application message. The captured client fragment group has no missing data.

That client group carries application chunks of 500 and seven bytes. The first compressed fragment occupies 368 wire bytes and expands to a 506-byte body (sequence, total length and chunk); the second occupies 15 wire bytes. Thus the client's fragment policy is not the legacy server's simple rule of splitting application data into 512-byte chunks. Preserve the observed header overhead and compression distinction when implementing a sender.

This behavior matches the recovered continuation-fragment check in `LibSOE/SOE.Core/SOEDataChannel.cs`: its comparison uses the previous fragment number in a way that rejects the next incrementing fragment. The trace establishes the false out-of-order notice; the source comparison supplies the probable cause. A successful world load therefore does not validate the legacy implementation's reliable receive, fragment, retransmission or ordering logic. The Rust implementation should retain independent sequence/fragment checks rather than copy this defect.

## User-approved walking check

The later 020419Z run records approved input in `ui-checks.json`: focus the game, close its news modal at client coordinates 967,44, and hold W for 1,200 ms. Its screenshots `world-before.png` and `world-after-walk.png` show the rendered world; resized `_small.png` copies are available for review. The UI check includes screenshot hashes.

The server logged **12 distinct position updates**, changing from approximately `(-1414.643, -27.63482, 351.5564)` to `(-1411.163, -29.97115, 342.4458)`, a maximum horizontal displacement of **9.75 world units** from the first position. This is actual input-driven walking evidence, beyond the single spawn-position packet in the earlier no-input run. It was performed by the parent session; this packet-analysis step launched no processes and sent no input.

## Limits and next acceptance conditions

- The baseline uses the supplied compiled legacy walking emulator. Rust currently provides tooling and transport components, not the running world service.
- Save/restart/reload persistence remains unverified. The UI check explicitly records `persistence_verified: false`.
- Terrain/asset warnings, including `SG_Dirt` warnings, remain in the client logs. Rendering one scene and walking do not prove exhaustive asset coverage or collision correctness.
- Quests, combat, jobs, social systems, minigames, TCG and other activities have not been validated by this baseline.
- Packet loss, retransmission, duplicates, rollover and longer sessions require separate transport tests. The 90-second sequence has no captured reliable gaps, but does expose the legacy fragment defect.

Use this working client/emulator baseline and private byte oracle to verify the next Rust gateway slice. Preserve the distinction between genuine restored behavior, canned emulator state, recovered content and unimplemented systems.

## Subsequent Rust acceptance

The limits above describe the original managed-emulator baseline. The subsequent Rust gateway passed actual-client login, world entry, walking, intentional reliable-packet loss recovery, saving, and restoration after restarting both Rust and the client. The full evidence is in [[Rust Gateway Milestone]]. The original baseline remains a separate reference; a screenshot of starter quests or inventory still does not prove their gameplay behavior. Client texture/image warnings persist.
