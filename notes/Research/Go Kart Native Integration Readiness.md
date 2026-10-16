# Go Kart native integration readiness

Read-only readiness extraction, October 9, 2026 (UTC). Task: locate the existing kart artifacts,
recover the exact native launch order and bindings, and state the minimum production wiring plus the
blockers for launching the original native kart race with a legitimate existing character. No
source, content, save, UI, release or process change; no client was launched. Nothing here is a
playability or historical-fidelity claim.

Owned artifacts:

| Path | Role |
| --- | --- |
| this note | inventory, launch order, wiring, blockers |
| `lab/reports/go-kart-native-readiness-20261009T143531Z/proof.json` | pinned proof, sha256 `52909fe6327b0f20ab6b0fbf49e33ebaaf2d467a9cb1026db8c643b38bcc9865` |
| `lab/reports/go-kart-native-readiness-20261009T143531Z/trace-go-kart-launch-order.py` | small reproducer over the existing timestamped index |
| `lab/reports/go-kart-native-readiness-20261009T143531Z/trace-run.log` | run output |

## Existing artifacts (nothing new was researched)

| Artifact | State | Key fact |
| --- | --- | --- |
| `prototype/osfr-tools/src/kart_racing.rs` | unregistered module, 13 builder tests + 12 review tests green | wire decoder, `LobbyGameDefinition::encode` (community subset only), `RaceSession` authority |
| `prototype/osfr-tools/tests/kart_racing.rs`, `tests/kart_review.rs` | `#[path]` harness | the module is not in `src/lib.rs` and not referenced by `src/gateway.rs` (proof `current_wiring`) |
| `notes/Research/Kart Racing Restoration.md` | accepted inside its own bounded scope | wire schema, identity data, native gates |
| `notes/Reviews/Kart Independent Review.md` | repair follow-up closed findings 1-8 | remaining gates: native 102/2 framing, subtype mapping, geometry, member binding, pricing |
| `lab/reports/kart-racing-20261009T0158Z/` | analysis bundle | `kart-racing-wire-schema.json`, `kart-racing-session-trace.txt`, community lobby oracle (80 B `e088a1f9…`) |
| `lab/reports/kart-review-20261009T024747Z/`, `kart-review-repair-20261009T031700Z/` | review + repair bundles | capture re-check: 22,389 opcode-33 bodies, 0 length mismatches |
| `lab/reports/original-minigame-packets/racing-application-index.json` | timestamped index, sha256 `eebcbfafbe6dd3134af6dcd2fe0f3135aba87302c68d7ac33347a99d7b930c2b` | 25,259 application frames, 95.07 s window; the launch order below comes from here |
| `lab/references/OpenSourceFreeRealms/ofrserver/Packets/packets_racing_minigame.pcapng` | pinned capture | original racing traffic, all CRCs valid under the existing model |
| `lab/decompiled/kart-racing-20261009T0158Z/vehicle-race-classes.json` | Ghidra RTTI inventory | 62 classes, incl. `BaseVehicleRacePacket`, `BaseLobbyGameDefinitionPacket`, `ApiLobbyGameOption` |
| `notes/Research/Pieces/015 Minigame launch catalogue.md` | accepted catalogue | kart rows are TYPE 10/11/13/19/20, native-client; 190 `vehiclerace` tokens; lobby ids 3052/3053 |
| `notes/Research/Pieces/018 Minigame native test routes.md` | route catalogue, gate default-off | `kart_racing`/`kart`/`racing` is a code/test candidate driven by `--test kart_racing`; no launch route |
| `lab/manifests/rust-asset-catalog.tsv` | 24-pack catalogue | `kt_kart07_body_bbe.cdt`, `kt_kart08_*`, DME/ADR parts, racing icons, `derbyResults.gfx`, `derbyCrown.gfx` |

## Kart authority as it exists today

The only kart authority is the reconstructed server-side `RaceSession`: ordered checkpoint ids,
lap/best-lap accounting, one-shot claim derived from a finished, fully-counted, explicitly priced
run. It has no physics, no position and no spatial corroboration; `observe_racer_state` accepts the
recovered payload sizes as opaque diagnostics. The gateway binding (which member owns which session
id, who may send checkpoints) does not exist yet. The client's own position-like values live in
subtypes 5/6/14 (bulk, layout undecoded) and 15/17/7 (three `f32` that look like coordinates); none
of them is trusted by the module. A readiness statement must therefore call the race
"server-arbitrated by reconstructed checkpoint ids", not a faithful recreation of the original race
authority.

## Lobby and protocol bindings

Community naming (original server enum dump, `PacketCodes.cs`): `BaseLobbyGameDefinitionPacket = 102`,
`BaseMatchmakingPacket = 141`. The racing stream is the tunneled inner opcode **33**
(`BaseVehicleRacePacket` family) under outer opcode 5/6; the current `gateway.rs::tunnel` already
accepts outer 5|6 and forwards unknown inner opcodes as `Event::Application`, so 33/102/141 can
reach `gameplay.rs::handle_in_world` without transport work.

The pinned community lobby writer (`Sanctuary`/`FreeRealms-Legacy` `LobbyGameDefinition.cs`) writes
`count, key, Id, Type, Unknown, NameId` per entry and then four zero counts; its handler replies to
102/1 with one test row `(1, 1, 37, 3030)`. The captured 102/2 is a **different, larger framing**:
after the 11-byte tunnel header it is `[u32 remaining length 2378][u32 row count 6][row0 =
(1, 1, 1, 37, 3030)]` followed by further blocks; row 1 is `(2,2,2,0,0)`. Row 0 is the Kart Race
entry and matches the community `// 1 - Kart Race` comment; the full six-row catalogue and the
trailing blocks are not decoded, so the community encoder is a subset and must not be shipped as the
live 102/2 response.

There is no community implementation of 141 (`BaseMatchmakingPacket`); only the enum name and the
two captured pairs exist.

## Exact native launch order (timestamps from the preserved index)

| t (s) | Flow | Inner | Subtype | Bytes | Meaning (recovered) |
| --- | --- | --- | --- | --- | --- |
| 0.000 | C→S | 102/1 | - | 11 | lobby game definitions request |
| 0.210 | S→C | 102/2 | - | 2393 | definitions: 6 rows, row0 = Kart Race `(1,1,1,37,3030)` |
| 1.034 | C→S | 33/9 | 9 | 27 | race entry: u64 session id `91ca73956acc524b`, `4ac46a2a`, `01000000` |
| 1.563 | S→C | 33/34 | 34 | 95 | pre-race block: `03000000 01000000 0000803f 0000803f` + zeros |
| 1.588 | S→C | 33/11 | 11 | 6177 | kart config: `u32 22` + `kt_kart08_body_bbe.cdt` + `f32 400.0` + full block |
| 1.588 | S→C | 33/5 | 5 | 96 | racer stream (bulk), starts with `0000803f` |
| 1.588 | S→C | 33/14 | 14 | 84 | progress stream: session id, `ffff7f7f` (FLT_MAX) slots, `03000000` |
| 1.662 | C→S | 33/2 | 2 | 23 | client join: u64 session id + `02000000` counter |
| 2.029 | S→C | 33/3 | 3 | 15 | `01000000` |
| 2.029 | S→C | 33/1 | 1 | 15 | `03000000` (counters advance 3..9 over the session) |
| 4.060 | S→C | 33/4 | 4 | 15 | `07000000` |
| 7.820+ | C→S | 33/6 | 6 | 96 | client racer stream (bulk, 4,679 frames) |
| 14.899+ | S→C | 33/15 | 15 | 43 | id + 3 `f32` (position-shaped) + `1.0, 1.0` + `1324` |
| 19.420+ | C→S | 33/7 | 7 | 35 | u64 session id + 3 `f32` + `1.0` |
| 22.921+ | C→S | 33/17 | 17 | 35 | same shape as 7 (33 frames) |
| 23.043+ | S→C | 33/16, 33/18 | 16, 18 | 15 | state values (`1`; `5,3,1,4`) |
| 26.293 | C→S | 33/22 | 22 | 19 | session id |
| 26.459 | S→C | 33/23 | 23 | 11 | empty payload |
| 74.5 | - | - | - | - | last racing frame; the stream simply stops |
| 75.153 | C→S | 141/1 | - | 19 | matchmaking after the race |
| 84.979 | S→C | 141/2 | - | 413 | matchmaking response, embeds the session id |
| 84.981 | S→C | 102/2 | - | 2393 | lobby definitions again |
| 94.893 / 95.066 | C→S / S→C | 141/1, 141/2 | - | 19 / 413 | second matchmaking pair |

The race is also launched as an **encounter**: the same capture carries `41/114` details (encounter
`917566`, instance `1008`, difficulty/ProfileType fields) and `41/2` player-enter with trailing byte
`0`, plus `41/107/108/109/120/121/132`. The production encounter lifecycle already emits 41/114 and
41/2 for the reconstructed cave, so that half of the launch path exists and is proven.

## Minimum production wiring to launch the original native kart

1. Register `kart_racing` (lead-owned `src/lib.rs`) and route inner opcodes 102/141/33 from
   `gameplay.rs::handle_in_world` into the module. The tunnel path needs no change.
2. Answer inner **102/1** with a **102/2** body whose framing matches the client parser (length
   prefix, row count, rows; row 0 = Kart Race). The recovered row 0 and row 1 are the only proven
   rows; the remaining catalogue rows and trailing blocks are unrecovered, so this is the first
   blocker for a real lobby.
3. On the client's inner **33/sub9** (session id), open a `RaceSession` bound to the existing
   character/member (gateway-owned binding) and emit **33/sub34**, **33/sub11** (kart body asset +
   config block) and **33/sub5/sub14** initial state. Answer **33/sub2** joins; drive **33/sub1/3/4**
   counters. Treat **33/sub6/7/17** and subtypes 5/14 as observations only.
4. Reuse the existing encounter lifecycle for the race encounter (`41/114` + `41/2`) exactly as the
   cave uses it; the capture's race encounter ids are `917566`/`1008`.
5. Bind the race to the speedway zone/geometry. The capture's position-shaped values (first
   `33/15` sample `(642.19, 70.50, 533.80)`, `33/17` `(645.16, 72.21, 533.13)`) match none of the
   five speedway POI coordinates (Wildwood `-2267.6,-16.3,585.4`; Thunder Falls `-915.6,-39.1,1203.7`;
   Frost Ridge `-233.0,1.2,-50.4`; Tangletrack `-273.8,-58.8,1951.9`; Sandstorm `34.5,-14.8,-1188.9`),
   so the track's zone instance, spawn transform and geometry id stay **unrecovered**.
6. Reward through the lead-owned economy layer only from the module's priced, finished run; the
   placement/amount mapping is unrecovered.

Blockers in dependency order: (a) the full 102/2 framing, (b) the subtype-to-class mapping for the
33 stream, (c) the speedway zone/spawn/geometry binding, (d) the race-finish/results flow (the
stream stopping is not a result message; 141's semantics are only enum-named), (e) gateway member
and session ownership, (f) reward numbers. None of these is invented here.

## Actual test gates (what can and cannot be tested natively)

| Gate | Status | Exact observable |
| --- | --- | --- |
| Launch | testable once (a)+(c) exist | existing character opens the lobby, sees a Kart Race row, selection produces 33/34+11; the client loads the named kart asset and does not crash |
| Control | testable | client sends 33/6/7/17; server receives and reports them; no server-side position authority is claimed |
| Race finish | **not definable today** | no finish/results message is recovered; the 33 stream ends after 74.5 s and the only follow-up is a `141` matchmaking pair + a fresh 102/2 |
| Reward | **blocked** | no pricing data and no finish binding; the module returns `Unpriced` for any track without an explicit schedule |
| Restart | partially testable | a second 102/1 + 141 pair follows the first race in the capture, so re-entering the lobby is observed; a full restart/reward loop cannot be asserted until the finish flow exists |

Asset/movie verdict: kart racing has **no movie**. It is a native in-world controller; the catalogue
carries kart bodies and parts plus small UI pieces (`derbyResults.gfx`, `derbyCrown.gfx`, racing
icons) but no `game_racing.gfx`/`game_kart*.gfx`. The `karts1.gfx` size quoted in
`notes/Research/Kart Racing Restoration.md` is **not reproducible** from the pinned catalogue
(recorded here as a discrepancy, not resolved).

## Next builder assignment (exact owned paths)

Extend only `prototype/osfr-tools/src/kart_racing.rs` and its owned tests
`prototype/osfr-tools/tests/kart_racing.rs` + `prototype/osfr-tools/tests/kart_review.rs`, update
`notes/Research/Kart Racing Restoration.md`, and add one private bundle under
`lab/reports/kart-native-<utc>/`. Shared integration (`src/lib.rs`, `src/gateway.rs`,
`src/gameplay.rs`, content, save, UI, release) stays lead-owned; the exact wiring list above is what
the lead must apply, and the first testable milestone is gate "Launch" after blockers (a) and (c)
close.

## How this was derived

The reproducer reads only the existing timestamped index and the already decoded `racing-*.bin`
bodies, joins them by sha256, asserts the documented launch precedence (102/1 before 102/2; 33/sub9
first; 34/11/5/14 before the first sub2), asserts the 102/2 framing fields, records the first bytes
of every subtype, and checks the current wiring literals in `lib.rs`/`gateway.rs`. It writes
`proof.json` with `write_exact` semantics; the recorded run is `trace-run.log`.

Cross-links: [[Research/Kart Racing Restoration]], [[Reviews/Kart Independent Review]],
[[Research/Pieces/015 Minigame launch catalogue]], [[Research/Pieces/018 Minigame native test routes]].
