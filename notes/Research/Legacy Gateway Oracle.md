# Legacy gateway byte oracle

The supplied compiled server now has a reproducible, local login/ready byte oracle. A synthetic UDP client completed its session, requested login, then sent client-ready. **This is evidence of the supplied walking emulator's behavior; it is not an original-client login or gameplay test.** The community Sanctuary implementation provides the richer application reference and differs materially from this older server.

## Capture and reproduction

Run from `%USERPROFILE%\Documents\Projects\CartoonMMO\project`:

```powershell
& '%USERPROFILE%\.cache\codex-runtimes\codex-primary-runtime\dependencies\python\python.exe' scripts\collect-legacy-oracle.py
```

`--skip-ready` captures only session and login. Each invocation creates new output and a new server copy. It preserves the input files, selects an available test port, changes only the copied Config.json port, uses fresh logs, launches the exact copied executable hidden, and terminates that process in a `finally` block. Startup is limited to 20 seconds and each receive phase to 12 seconds with a 1.5-second idle limit. These limits do not promise successful capture under load; failure is recorded rather than silently accepted. The legacy executable binds all IPv4 interfaces; the collector sends only to loopback and uses a transient port. No client executable is started.

The successful capture is:

`%USERPROFILE%\Documents\Projects\CartoonMMO\lab\captures\legacy-gateway-20261008T014620082859Z`

Its copied executable/configuration and fresh server logs are under `lab/runs/legacy-gateway-20261008T014620082859Z`. The captured server PID was 10244 and was stopped. `manifest.json` records its executable SHA-256, directory, port, process result, every captured frame and every reconstructed message. Raw packet and application data stay in the lab, outside the project repository.

| Result | Count |
| --- | ---: |
| Captured transmit/receive frames, including session and ACKs | 1,259 |
| Received post-session datagrams, each independently CRC-checked | 1,253 |
| Received zlib-compressed datagrams | 1,237 |
| Complete application replies | 46 |
| Reconstructed application bytes | 627,566 |
| Login replies | 8 |
| Client-ready replies | 38 |

The collector checks transport sequence continuity, fragment declared lengths, zlib termination, expansion limits, and tunnel payload lengths. No gap or incomplete message occurred. `independent-recheck.json` records rechecking all 1,253 frames, then rejecting a CRC-corrupted variant of every frame. Invalid compression flags and incomplete zlib streams were also rejected. The server's own receive-CRC loop is not trusted for this validation.

Two diagnostic captures are retained alongside the successful run: the first exposed copied historical log lines falsely satisfying the readiness check; fresh logs fixed that. The second queued an ACK per outbound fragment, delaying the ready request behind hundreds of 13 ms server processing cycles. The final collector sends one end-of-phase ACK. The legacy server only logs ACKs and sends an entire stream without a transmit window, so this choice establishes no real-client ACK/window semantics.

## Verified wire framing

The initial request/response match [[SOE Transport Notes]]. For subsequent data in this negotiated session:

```text
transport opcode u16 BE
compression flag u8 (0=plain, 1=zlib)
body (plain, or one zlib stream)
CRC low 16 bits u16 BE
```

The checksum equals standard CRC-32 of `seed.to_le_bytes() || transport_header_and_body`, with seed `1884358976`. It includes the opcode and compression flag, excludes the CRC trailer, and applies to the compressed bytes before inflation. This identity matched every actual outbound datagram; the collector uses Python's zlib CRC and inflate implementations independently of LibSOE.

Data opcode 9 has a body of `sequence u16 BE || application bytes`. Fragment opcode 13 has `sequence u16 BE || total_application_length u32 BE || first_chunk` for the first fragment and `sequence u16 BE || next_chunk` for the remainder. The legacy sender splits application bytes into chunks of at most the negotiated 512 bytes; the total length includes the application opcode. Its first fragment therefore has six header bytes before the chunk, and subsequent fragments have two. This is a description of these captured sends, not a maximum-UDP-payload rule to copy into a new server.

Application opcodes are u16 **little-endian**. The minimal legacy login was exactly `01 00`: its handler ignores all subsequent login fields. Client-ready was `05 00 01 02 00 00 00 0d 00`: outer tunnel opcode 5, reliable boolean true, little-endian payload length 2, nested opcode 13. Both were sent through opcode 9 with independently generated CRCs.

All 46 captured replies use this application envelope:

```text
05 00                  outer gateway tunnel opcode
01                     reliable boolean
payload_length u32 LE  includes nested opcode
nested_opcode u16 LE
nested_payload...
```

The `frames` manifest array stores relative `path`, `direction`, `phase`, `length`, and `sha256`; decoded received frames additionally have `opcode`, `crc_verified`, `compressed`, `expanded_body_length`, and `sequence` or `ack_sequence`. The `messages` array stores `path`, `phase`, first/last sequence, length, SHA-256, outer opcode, tunnel flag, inner length and inner opcode. `messages/*.bin` contains the entire application envelope above, with no transport header, sequence, compression flag or CRC trailer.

## Initial application sequence

Lengths here include the seven-byte outer tunnel envelope. The source method names identify observed legacy behavior, not complete restored systems.

| Order | Nested opcode | Application bytes | Observed content |
| --- | ---: | ---: | --- |
| 1 | 41 | 20 | Encounter overworld-combat configuration |
| 2 | 2 | 10 | Successful login reply, wrapped by legacy server |
| 3 | 166 | 14 | Initialization parameters (`live`) |
| 4 | 43 | 46 | Zone details, legacy name `FabledRealms` |
| 5 | 144 | 26 | Client game settings |
| 6 | 35 | 263,908 | Item definitions, subtype 37; fragments 5–520 |
| 7 | 193 | 71 | Announcement data |
| 8 | 12 | 203,850 | Supplied predefined character; fragments 522–920 |

Ready then sends quick-chat data (98,815 bytes), point-of-interest definitions (10,644 bytes), 23 NPC/warp-object messages, hitpoint/mana/ability/effect state, journal and coin-store data, welcome screen, zone-done opcode 14, preload-done and stat-update packets, and MOTD. These 38 application replies span sequences 921–1250. NPC GUIDs come from `RandomGUID`, so a repeated capture need not match those messages byte-for-byte. The item and character definitions originate in the copied `Customize` JSON files; this server does not build a fresh persisted character for the synthetic login.

Evidence in recovered files under `lab/decompiled/server`:

- `Gateway/Gateway.Login/LoginManager.cs:49`: ignored login payload and exact send order.
- `Gateway/Gateway.Login/LoginManager.cs:65`: ready response order.
- `Gateway/Gateway.Login/LoginManager.cs:371`: tunnel serialization.
- `Gateway/Gateway.Player/PlayerCode.cs:83`: self data from Customize JSON.
- `LibSOE/SOE.Interfaces/SOEWriter.cs:219`: compression threshold and CRC serialization; body over 100 bytes is compressed.
- `LibSOE/SOE.Interfaces/SOEWriter.cs:278`: application fragmentation.
- `LibSOE/SOE.Core/SOEDataChannel.cs:181`: fragment transmission.
- `LibSOE/SOE.Core/SOEProtocol.cs:260`: seeded checksum routine.

## Community comparison and remaining behavior

At the pinned Sanctuary revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, `src/Sanctuary.Packet/ClientGateway/PacketLogin.cs` reads ticket string, character GUID u64, client-version string, and another string, requiring the entire packet to be consumed. `PacketLoginHandler.cs` checks version, parses the ticket, initializes the cipher and loads a database character. It sends PacketLoginReply as a top-level gateway packet. The supplied older server's two-byte permissive login and tunneled login reply must therefore remain labeled as legacy behavior rather than a complete protocol specification. Sanctuary's tunnel class agrees with the observed boolean plus length-prefixed payload format. The checked-out source is a reference; Sanctuary has not been built or tested here.

For walking after readiness, the legacy dispatcher handles time sync 52, teleport request 90, safe teleport 122, inventory 42, mounts 168, chat 15/67, position 125 and camera 126. Time sync returns a current timestamp; teleport handlers send client update 38/subtype 12; movement logs positions and can send area/POI updates. The shown movement path does not save character coordinates. Client-finished-loading opcode 10 has no explicit handler in this dispatcher. A world-tunnel opcode 6 also accepts friend-position command 26/subtype 21 and telemetry 194. This static inspection does not establish that any of these paths work with the supplied client.

The next acceptance step is an **actual supplied-client** connection to the copied walking emulator: observe its complete login fields, session/application encryption, packet bundling, ACK pattern, world-load completion, movement and disconnect behavior. A real-client trace can resolve the legacy/Sanctuary differences and reveal additional messages absent from these synthetic requests. The oracle is suitable for testing framing and comparing restored application serialization; canned reply replay alone would not implement gameplay, authentication, persistence or reliable transport.
