# World and Instances

Status: W20 isolated coordinator candidate. It is not registered in `lib.rs` or routed through `gateway.rs`; tests use `#[path]` while the lead owns integration.

## Model

`src/world.rs` indexes the validated `ContentDb` by zone, zone NPC rows, and dungeon ID. It owns:

- One shared public instance per zone. (R) Its instance ID equals the zone ID so the existing zone-0 startup identity remains stable.
- Private child instances tagged as a dungeon, encounter, or house. Their internal IDs are separate from the client-visible zone ID. The parent is recorded; children load the NPC spawn set for their content zone.
- Player membership, last pose, and a per-instance client-ready set.
- Empty-private-instance cleanup after the configured grace interval. (R) Public instances remain cached. Private IDs are positive, process-local, sequential values; they are not persistent or cluster-safe.

`enter_dungeon` resolves `Dungeon.zone_id`, creates a private child, and stages the entry sequence before moving the player. `enter_instance` builds every packet before changing membership. A serialization/content error therefore leaves the prior instance, pose, ready state, and memberships intact. `leave` removes the player and starts the empty timer. `collect_empty_instances(now_ms)` removes expired private leaves before their empty parents.

## Entry and readiness sequence

For a first entry outside zone 0, and for any transfer, the framed packet order is:

1. Opcode 31 begin-zoning (`zone_transition::encode`).
2. Opcode 43 zone details (`startup_messages::build_zone_details`).
3. One 35/2 AddNpc (`startup_world::build_add_npc`) per `ContentDb.npcs` row for the instance's zone.
4. The returned `EnterSequence` sets `awaits_client_ready`; the caller gates gameplay until its ready/load callback calls `mark_zone_ready`.

Both opcode 31's `Id` and opcode 43's `zone_id` carry the content zone ID. The existing gateway copies `ZoneTransition.id` into `CharacterState.zone_id`; private instance routing stays in `World` membership.

(R) The first zone-0 entry follows the current login path: it sends 43 details without a new opcode-31 transition. The current gateway's initial-zone packet is `initial_zone` in `gateway.rs`; its zone type is 2. The synthetic zone-0 fixture checks exact 43-frame parity with that serializer path. It does **not** reproduce the complete opening initialization burst.

`mark_zone_ready` records the member and returns true when every current instance member has reported ready. (R) This all-members barrier is local policy; no original multi-session server policy was recovered. The method emits no minigame packets. The existing `EncounterEngine::on_zone_ready` owns its current encounter burst, including 41/114, 41/107 and 41/2 ordering. The Minigame Framework guide identifies 41/108 as the encounter entrance request, 41/2 as the encounter start handler, and 41/107 as encounter-zone-ready traffic. Do not route the same readiness event to both packet owners after integration.

## Evidence and provenance

Recovered client facts used:

- Opcode 31 field order is documented in `src/zone_transition.rs`: the client reader is `FUN_008e4ae0` in `lab/decompiled/client/FreeRealms/pseudocode.c:1029104`, reached from the opcode-31 switch at `:1108664`. `python -I scripts/native_map/nmq.py dis 008e4ae0 180` showed the counted string reads and two `ReadFloat4` calls at `008e2410`. The full order in the existing codec is opcode, scene string, type, position float4, rotation float4, sky string, tutorial/unknown bytes, ID, update radius, geometry and two booleans.
- Existing `zone_transition.rs` records envelope flag `1` in every preserved opcode-31 capture; the new sequence uses that captured compatibility value.
- 35/2 AddNpc uses confirmed `PlayerUpdatePacketAddNpc_ReadBody` at `00921a10`. `notes/Research/Startup World Content.md` records its field walk and helper readers. `python -I scripts/native_map/nmq.py dis 00921a10 180` confirmed the initial scalar reads and reader calls. The native reader proves widths/order, not all server meanings.
- Opcode 43 uses existing `startup_messages::build_zone_details`. `PacketSendZoneDetails` constructor `008cebc0` and the current gateway field walk are documented in `notes/Research/Startup Message Builders.md`; a native receive reader is **not** mapped. No new reader claim is made for 43.
- Encounter references come from `notes/Build Guides/Gameplay/Minigame Framework.md`: 41/2 handler `009b9550` and encounter dispatch at `00aa36c0`; the same guide labels 41/107 zone-ready and 41/108 entrance request. Existing `encounter.rs::on_zone_ready` supplies the current ordered server burst. This module does not infer additional packet fields.
- `ContentDb` zone and NPC fields are the schema in `src/content.rs` and `notes/Research/Content Data Model.md`. That note distinguishes native 35/2 values from reconstructed role, zone, and quest crosswalks.

Reconstructed policy and crosswalks `(R)`:

- One public instance per zone; public instance ID equals zone ID. Private IDs are in-memory only.
- The nonzero first-entry/transfer opcode-31 path, default zone type 2, tutorial false, empty sky, geometry 0, update radius 0, unknown flags 0, and `wait_for_zone_ready=true` when optional fields are absent or not otherwise sourced.
- `override_update_radius=false`, neutral `WorldPose::ORIGIN`, and keeping an empty private parent until its children are removed.
- Missing NPC position-w and scale default to 1.0. Heading rotates to a z-axis quaternion by the conventional half-angle formula. The content schema does not establish that conversion.
- The content schema omits much of the 35/2 row. The packet crosswalk leaves unknown fields zero/empty, including the textual NPC name; roles do not imply vendor behavior or a recovered interaction radius.
- All-current-members readiness and configured private-instance grace are local server policies, not recovered original rules. House ownership/access checks are outside this coordinator.

## Current zone-0 boundary

`gateway.rs::templates_for` substitutes current zone details in startup slot 03 and still replays static captured 35/2 messages in slots 10–32. On `Event::Loaded`, `gameplay.rs::initialize_world` sends opening-specific initialization; `opening_world.rs::OpeningWorld::initialization` builds its dynamic actor packets and relevance update. `encounter.rs` continues the selected cave entry/return path. Kart entry uses the separate `zone_transition` codec and kart lifecycle.

The W20 parity test freezes only the representable zone-0 43 details frame against the current gateway serializer with synthetic zone metadata. `ContentDb::NpcSpawn` lacks the full startup 35/2 fields, and opening actors come from their separate opening content/template. Full zone-0 packet-for-packet parity is still open; do not replace the captured startup or `OpeningWorld` path based on this test.

## Worker method and verification

- Read `AGENTS.md`, `MMO Worker Contract.md`, `Build Ledger.md`, `Zones and Terrain.md`, `Content Data Model.md`, and `Minigame Framework.md`, plus the requested source modules. The World guide's narrative is still a placeholder.
- Applied Ponytail: reused `ContentDb`, transition and startup builders; added no dependencies and touched no shared module. Applied Caveman to progress prose; saved evidence remains explicit.
- Applied `reverse-engineering` narrowly by disassembling the existing 31 and 35/2 readers above. `game-recon`, `game-research-websearch`, and `asset-pipeline` were not needed: this slice uses existing client layouts and no installation, web research, or asset conversion.
- `RTK.md` requests normal commands first. PowerShell and `rg` output was usable; no `rtk proxy` retry was needed.

Commands and results:

```text
python -I scripts/native_map/nmq.py dis 008e4ae0 180
python -I scripts/native_map/nmq.py dis 00921a10 180
```

Both returned disassembly; reader interpretation is bounded as described above.

```text
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W20'
..\..\scripts\cargo.ps1 test --test world
```

An early invocation reported unresolved-crate errors for `osfr_tools` in existing `src/account.rs` and `src/account_store.rs`; those shared files were not edited. The final invocation, with `extern crate self as osfr_tools` present in `src/lib.rs`, passed: **7 passed, 0 failed**. It reports the existing dead-code warning in `housing.rs::Lot::instance_index`.

Scoped formatting commands (only owned Rust files):

```text
$rust = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe'
& $rust --edition 2024 src\world.rs tests\world.rs
& $rust --check --edition 2024 src\world.rs tests\world.rs
```

Both passed. Before the final seventh test was added, a scoped fallback compiled `tests/world.rs` directly with the private Rust 2024 toolchain and Cargo-built dependency rlibs, then ran `lab/build/W20/world-tests.exe`: **6 passed, 0 failed**.

```powershell
$env:CARGO_MANIFEST_DIR = (Get-Location).Path
$target = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W20\debug\deps'
$serde = (Get-ChildItem -LiteralPath $target -Filter 'libserde-*.rlib' | Select-Object -First 1).FullName
$json = (Get-ChildItem -LiteralPath $target -Filter 'libserde_json-*.rlib' | Select-Object -First 1).FullName
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustc.exe' --edition=2024 -A dead_code --test tests\world.rs -L "dependency=$target" --extern "serde=$serde" --extern "serde_json=$json" -o '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W20\world-tests.exe'
& '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W20\world-tests.exe'
```

No crate-wide formatting or native-client run was performed.

## Lead integration steps and open gaps

1. Register `world` in shared `lib.rs`; keep this worker candidate isolated until review.
2. Route login entry and opcode-31 transfers through `enter_public` / `enter_instance`. Commit character zone and pose only after packet construction succeeds. Route disconnects through `leave` and schedule `collect_empty_instances`.
3. Feed the actual client ready/load callback to `mark_zone_ready`. Keep `EncounterEngine::on_zone_ready` as the single owner of its current 41/114, 41/107, 41/2 and related burst, or move that ownership as one reviewed change.
4. Add the cave/dungeon's validated `ContentDb` zone/NPC rows. The current fixture's dungeon is synthetic and only tests parent/child mechanics.
5. Expand the zone-0 content crosswalk to the full static AddNpc rows and opening actor data before claiming full opening parity. Add an independent captured-byte oracle outside Git where project policy allows.
6. Decide durable ID allocation, multi-process ownership, house access policy, NPC interactions and broader reconnect behavior separately; all remain absent or `(R)` here.
