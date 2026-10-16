# Developer Teleport

`realm goto` is a local developer tool for the running Rust gateway. It relocates the single supported character to a named zone-0 actor using the original community location-update packet, so debugging no longer costs a full walk across Sanctuary or Farnum's Farmstead. It is off unless the launcher passes `--dev-tools`, and it changes nothing except the character position and facing.

This is a local development command, not recovered original content. The packet it sends is recovered community evidence; the command syntax, the target lookup and the arrival standoff are ours.

## Wire evidence

Pinned Sanctuary `Sanctuary.Game/ChatCommands/TeleportCommand.cs` teleports by sending `ClientUpdatePacketUpdateLocation` with `Teleport = true`. The writer chain is `Sanctuary.Packet/BaseClientUpdatePacket.cs` (base opcode38), `Sanctuary.Packet/BaseClientUpdatePacket/ClientUpdatePacketUpdateLocation.cs` (sub-opcode12) and `Sanctuary.Core/IO/PacketWriter.cs`, which writes a full `Vector4` position, a full `Quaternion` rotation, then `bool Teleport` and `byte Unknown`.

Field values come from the same pinned source: `Sanctuary.Gateway/Handlers/PacketZoneTeleportRequestHandler.cs` builds `new Quaternion(rotationZ, 0f, rotationX, 0f)`, so the stored rotation is a unit facing vector with W=0, not a real quaternion. The captured self payload `lab/references/rust-gateway-reference/0007-login.bin` agrees at offsets164/180 (position W=1, rotation W=0). `PacketZoneSafeTeleportRequestHandler.cs` and `PacketWorldTeleportRequestHandler.cs` send the same packet shape.

`src/dev_teleport.rs` therefore emits exactly:

| bytes | field |
| --- | --- |
| `26 00 0c 00` | base opcode38 (`0x26`), sub-opcode12 (`0x0c`) |
| 4 x `f32` | position X,Y,Z,W=1 |
| 4 x `f32` | facing X,Y,Z,W=0 |
| `01` | `Teleport = true` |
| `00` | `Unknown`, the writer default |

The private harness `../lab/reports/developer-teleport-20261009T003739Z/oracle` compiles those four pinned files unchanged with the private .NET10 SDK and prints the serialized bytes:

```powershell
& "$lab/tools/dotnet/dotnet.exe" build <report>/oracle -c Release
& "$lab/tools/dotnet/dotnet.exe" <report>/oracle/bin/Release/net10.0/LocationWriterOracle.dll -2112.839 -56.52175 208.5168 0 0 -1
& "$lab/tools/dotnet/dotnet.exe" <report>/oracle/bin/Release/net10.0/LocationWriterOracle.dll -1054.135 4.629629 305.7209 -0.70710677 0 -0.70710677
```

Both vectors reproduce byte-for-byte and digest-for-digest in the ordinary Rust unit test `dev_teleport::tests::location_packet_matches_the_pinned_c_sharp_writer`: `0a26a9d0202c2a47fe161befa9fea66d8bccb1bc5e1a486408019a4b601b686e` and `ddd1827f84b7dcea7c2c3291d9e108aa2f4ee9e87e208316420600f2fb30e2c7`, both38 bytes. Pinned source hashes are frozen in `<report>/baseline-hashes.txt`. Native packet acceptance is the lead's remaining gate; this note claims writer fidelity only.

## Command contract

`realm goto <name or GUID>` and `realm goto list` are handled before the existing opening chat dispatch, and only while the character is in zone0 with finite coordinates. Matching is exact name, then unique GUID, then unique name prefix, then unique name substring; ambiguous or unknown targets reply with candidates and never move the character. `realm gotomae` is not a keyword: anything after `goto` must be absent or whitespace-separated, and such input falls through to the ordinary opening handler unchanged.

The arrival is `4.0m` from the actor at the actor's own Y, facing the actor. The candidate directions are sampled relative to the character's current facing vector in the order `0, +45, -45, +90, -90, +135, -135, 180` degrees, with angle0 on the approach side (the facing vector reversed, so a repeated `goto` stays put) and positive angles counter-clockwise in XZ. The first candidate that keeps the target at least `0.5m` closer than every other loaded actor in the same zone wins; if none clears the margin, the candidate with the largest margin wins, earliest sample first. Other-zone actors, the target itself and non-finite positions are ignored. That keeps the arrival inside the actor's12m interaction radius and on the actor's ground height; if the facing vector and the actor-to-character direction are both degenerate, the sampler starts at `+X`. It is a bounded standoff only: `realm goto` does not parse collision geometry and does not claim a verified floor.

The sampler exists because a direction-only standoff is not enough in the farm cluster. Mae sits at `[-2112.839,-56.52175,208.5168]` and the Cooking Table at `[-2109.244,-56.61908,210.265]`, about4m away with Gummyfruit, the Water Well and more Fire objects nearby. The old approach-direction arrival landed at `[-2108.856,-56.52175,208.88252]`: 4.000m from Mae but only1.436m from the Cooking Table, so the table owned the prompt. The margin sampler arrives at `[-2116.839,-56.52175,208.5168]`: 4.000m from Mae and7.794m from the table, and every one of the other27 loaded zone0 actors stays at least0.5m farther than Mae. That is prompt ownership, not a collision or reachability proof.

The relocation is position-only. `OpeningEngine::set_location_without_events` moves the remembered interaction origin without evaluating position events, so the command cannot grant a quest, discovery, reward, XP or item by itself, and the client can interact immediately without a dummy movement key. Entering the world and later client movement still run the ordinary rules and may award a discovery; that boundary is deliberate and tested separately. Relocation is refused while a crafting movie is open, while an encounter is active, and while the character is knocked out; every refusal leaves all state unchanged.

`Outcome.relocation` carries the staged position to `gateway.rs`, which applies it to the staged `CharacterState` before persistence, emits the writer-exact packet from that staged state, and sends it before the chat reply like the pinned handler. The position is therefore identical in the packet, the live session and the saved snapshot, and a malformed bundle child still discards the whole message.

## Verification

`src/dev_teleport.rs` ordinary unit tests cover the two compiled-writer vectors, exact/prefix/GUID/ambiguity/unknown resolution, the keyword boundary, non-finite target and character refusal, standoff distance/facing/finiteness with the approach and degenerate fallbacks, the occupied-first-sample skip, and the real-coordinate farm-cluster margin. `gameplay::developer_teleport` covers the crafting refusal. `tests/dev_teleport.rs` holds seven private integration tests: default-off, relocation plus progress preservation plus immediate interaction with a Queen negative control, the farm-cluster prompt-ownership regression, list/ambiguity/unknown/malformed/`gotomae` non-mutation plus idempotent replay, live encounter and knockout refusal, malformed-bundle atomicity plus save/load restart, and the location-setter state-equality boundary against a real `on_position` discovery.

```powershell
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --lib
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test dev_teleport -- --ignored --test-threads=1
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml
```

Evidence logs, the compiled writer oracle and the gateway flag smoke run are in `../lab/reports/developer-teleport-20261009T003739Z/`: `red-01-behavioral.log` (the failing packet assertion before implementation), `red-02-keyword-boundary.log` (scratch-reverted boundary hunk, both keyword tests red, then restored green), `red-03-standoff-cluster.log` (the farm arrival handing the prompt to the Cooking Table at1.436m before the sampler), `green-08-standoff-post-fmt.log` (24 ordinary library tests and seven private integration tests), `full-suite-final.log` (187 passed, 0 failed, 36 ignored across34 targets), `gateway-flag-smoke.log` (`"dev_tools":true` with the flag, `false` without), `release-gate.txt`, `final-hashes.txt` and `baseline-verify.txt`.

Status: code complete and packet-verified; the launcher forwarding already exists and native `--dev-tools` acceptance belongs to the lead. `../lab/reports/developer-teleport-20261009T003739Z/release-gate.txt` records that `target/release/osfr-gateway.exe` is still the accepted pre-candidate build (`ea297301...`) without the flag, that the frozen `accepted-pre-candidate-gateway.exe` copy is a separate path a rebuild cannot overwrite, and the exact release rebuild plus smoke command the lead needs before launching with `--dev-tools`. The command intentionally stays a local, zone-0, non-progression tool; it is not part of the restored original game.
