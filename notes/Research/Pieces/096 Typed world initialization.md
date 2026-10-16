# 096 Typed world initialization

Queue item 096. Acceptance: replace captured templates incrementally with unchanged-writer
vectors and exact native compatibility.

Status 2026-10-09 05:23 UTC: the 46-message initialization sequence is now registered as a typed
slot table. Twelve slots (the whole login phase plus the ready tail) have typed writers whose
encode output is **byte-identical** to the preserved capture; the other 34 slots keep replaying
captured bytes and name their recovered emitter. The writer proof is an unchanged-writer oracle
(`encode(decode(capture)) == capture`), not a native run. No shared or lead-owned file was
touched, the module is not registered, and no native acceptance is claimed.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_096.rs` | Tunnel frame plus typed encode/decode for 12 initialization messages, the 46-slot typed/captured registry, the captured-emitter table and `verify_captured` (new) |
| `prototype/osfr-tools/tests/piece_096.rs` | 9 hermetic schema and negative vectors plus the `#[ignore]` private unchanged-writer oracle; included by `#[path]` until the lead registers the module (new) |
| `scripts/pieces/piece_096.py` | Independent checker: derives the sequence from the recovered emulator call order, re-encodes every typed slot in Python and cross-checks the Rust table and attributions (new) |
| `tests/pieces/test_piece_096.py` | 7 stdlib tests, including three fail-closed negatives (tampered template, dropped writer, changed slot opcode) (new) |
| `../lab/reports/piece-096-20261009T052309Z/` | `piece-096.json`, RED/GREEN Rust logs, private oracle log, Python log, lib-test log, full-suite attempt, `summary.txt`, `commands.txt` |

Read-only evidence used: `lab/references/rust-gateway-reference/` (46 captured messages plus
`manifest.json`, sha256 `4608f6f0…`), `lab/decompiled/server/Gateway/` (recovered emulator source)
and the pinned community `Sanctuary` checkout (`lab/references/Sanctuary`, revision
`4f028dcc274f628b65ef665fd59b184f5f3d3e3e`). No captured byte is embedded in the repository.

## Original, recovered

1. **The tunnel frame is `u16 5 | u8 1 | u32 payload length | payload`.** The recovered
   `LoginManager.SendTunneledClientPacket` (sha256 `631fc8da…`) writes `SOEWriter(5)`, `bool true`,
   `AddHostUInt32(rawBytes.Length)` then the bytes. The pinned community
   `PacketTunneledClientWorldPacket` (sha256 `4e65d416…`) serializes opcode, `Reliable`, then
   `WritePayload` (u32 size + bytes); the two agree, and all 46 captures parse under it.
2. **The sequence and its emitters come from the recovered call order.** `HandleLoginRequest`
   sends `SendEncounterOverworldCombat`, `SendLoginReply`, `SendInitializationParameters`,
   `SendZoneDetails`, `SendClientGameSettings`, `SendPlayerUpdateItemDefinitions`,
   `SendAnnouncementData`, `SendSelfToClient` (slots 0-7). `HandlePacketClientIsReady` then sends
   quick chat (67), points of interest (57), entity adds, hitpoints (38), mana (38), the ability
   literal (36), the effect-tag pair (38, 38), three inline literals (209, 209, 165), the welcome
   screen (93), zone-done (14), preload characters (38), update stat (38) and the MOTD (87).
   The checker parses those handler bodies and requires the derived opcode list to equal the
   captured list slot for slot.
3. **The op35 entity block decomposes into the recovered per-function counts.** The decompiled
   bodies write 6, 14 and 3 messages for `SendPlayerUpdatePacketAddNpc`, `SendWarpStones` and
   `SendWarpWatchers` (sha256 `31322c2f…`), which matches capture slots 10-15, 16-29 and 30-32
   exactly (23 op35 slots). `UpdateStats.SendClientUpdatePacketHitpoints` has two mutually
   exclusive branches; only one message runs, which the checker records as an explicit override.
4. **Typed field order (all verified byte-identical against the capture).**
   `41 [u16 encounter id][u32][u32][bool]`; `2 [bool]`; `166 "live"\0` (null-terminated ASCII);
   `43 [u32-length ASCII name][u32 type][bool][bool][u32-length ASCII sky][bool][u32 id]
   [u32 geometry id][bool][bool]` (name `FabledRealms`, type 2, id 5, geometry 5 — matches the
   pinned `PacketSendZoneDetails`, sha256 `9c14a25e…`); `144 [u32 4][u32 7][u32 268][bool][f32 1]`;
   `193 [u8 2][u32 count]` then rows `[u32 ×6][ASCII][ASCII][u32 ×3][ASCII]`;
   `12 [u32 length][payload]` (payload is `ClientPcData.ReadFromJSON` output);
   `14` (opcode only); `38 [u16 26][bool]` and `38 [u16 7][i64 guid][i32 count]` then
   `(i32, i32, f32)` entries; `87 [ASCII title][ASCII body]`.
5. **Item definitions (op35 login) are count-prefixed rows of 41 recovered fields** in
   `PlayerCode.SendPlayerUpdateItemDefinitions` (sha256 `202576ff…`): eleven `i32`, two `bool`,
   two ASCII strings, three `i32`, two `bool`, seven `i32`, `bool`, `i32`, `bool`, ASCII string,
   `bool`, eight `i32`, then the `ClientItemStatDefinition` list (`key` + four fields + `bool`)
   and the `ItemAbilityEntry` list (four `i32`). The preserved payload decodes to 1383 rows and
   re-encodes to all 263 893 bytes exactly.
6. **Three messages are opaque literals with no recovered field structure** — the two inline
   op209 literals and the op165 literal in `HandlePacketClientIsReady`, the ability literal in
   `SendAbilityPacketSetDefinition`, the welcome-screen literal and the inline literal inside
   `SendClientUpdatePacketAddEffectTag`. They stay captured.

## Reconstructed (labelled, not historical)

* The Rust field names for values the recovered writer passes as literals (`unknown_a`,
  `unknown5`, `numbers`, `tail`, …); the recovered source carries no names for them.
* Reading the update-stat leading `i32` as an entry count. It equals the three following
  triples, but the recovered source does not name the field.
* The `captured_writer` attribution strings and the `RUST_WRITER_TOKENS` map are checker
  bookkeeping; the underlying order and counts are recovered.
* Per-entity identity of the op35 rows (which NPC, warp stone or watcher each row describes)
  needs the NPC catalogue and is not claimed.

## Verified locally

```
# RED before the module existed
scripts/cargo.ps1 test --test piece_096
  -> error: couldn't find file `tests\..\src\restoration\piece_096.rs`
# GREEN
scripts/cargo.ps1 test --test piece_096
  -> test result: ok. 9 passed; 0 failed; 1 ignored
# private unchanged-writer oracle
$env:OSFR_GATEWAY_REFERENCE=<lab>/references/rust-gateway-reference
scripts/cargo.ps1 test --test piece_096 '--' '--include-ignored' '--nocapture'
  -> test result: ok. 10 passed; 0 failed; "unchanged writer vectors: 12 typed, 34 still captured"
# checker (independent Python re-implementation)
lab/tools/wiki-env/Scripts/python.exe scripts/pieces/piece_096.py --output <report>/piece-096.json
  -> piece 096 evidence ok: 12 typed slots re-encoded byte identically, 34 still captured
lab/tools/wiki-env/Scripts/python.exe tests/pieces/test_piece_096.py -v
  -> Ran 7 tests ... OK
# library unit tests
scripts/cargo.ps1 test --lib -> 24 passed
```

Typed slots: 0, 1, 2, 3, 4, 5, 6, 7 (login) and 42 (zone-done), 43 (preload characters),
44 (update stat), 45 (MOTD). Every one is verified twice: once by the Rust decode/encode oracle
over the preserved capture and once by the independent Python encoder.

`scripts/cargo.ps1 test` (whole workspace) still fails to compile test targets that other workers
are adding mid-edit in this shared checkout (observed over the session: `piece_044`, `piece_047`,
`piece_068`, `piece_083`, `piece_099`); the failure text is kept in `full-suite-attempt.txt` and no
096 path is involved.

## Remaining gates

1. **Native acceptance (lead-owned).** No client run followed these re-encodings. The unchanged
   vectors prove byte equality with messages the native client already accepted; they do not by
   themselves prove a rebuilt stream is accepted end to end.
2. **Gateway integration (lead-owned).** `src/restoration/piece_096.rs` is not registered.
   `Gateway::load_reference` could call `verify_captured` for the 12 typed slots, and the typed
   messages could replace file replay once the lead chooses runtime values. The gateway's current
   `wrapped`/`initial_zone` pair duplicates two of these writers and should be replaced by
   `frame`/`TypedWriter::ZoneDetails` so the copies cannot drift.
3. **The other 34 slots** keep replaying captured bytes: quick chat, points of interest, the
   23-slot op35 entity block, hitpoints, mana, the ability/effect literals, the welcome screen,
   preload characters' neighbours and the two inline op209/op165 literals. Their emitters are
   named, but their payloads (resource-driven or opaque) need further recovery.
4. **Native compatibility for edited values.** The typed writers can now emit different GUIDs,
   zone ids, MOTDs or item rows; only the captured values are proven.
