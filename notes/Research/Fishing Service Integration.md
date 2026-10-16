# Fishing Service Integration

October 9, 2026. Builder scope: fishing only (queue item 003), following
[[Reviews/Delegated Fishing Readiness]]. This change adds a callable opcode 138
service and closes the two review findings (R1/R2) that had to be fixed before
any catch authority could exist. It does **not** register the service in
`lib.rs`, route the gateway, drive the native client, or grant coins,
experience, items or quest credit. `src/{gameplay,gateway,lib}.rs`, Cargo
manifests, shared content/saves, launcher and release files were not touched.

## Files and freeze hashes

- `prototype/osfr-tools/src/fishing.rs`: `eee55aed...` -> `e8266183...`
  (reviewed codec plus the R1/R2 result gate, `Session::cancel`, and the
  service-level `Error::AtCapacity`).
- `prototype/osfr-tools/src/fishing_service.rs` (new): `e1f19efa...`.
- `prototype/osfr-tools/tests/fishing_service.rs` (new): `4a77dc9a...`.
- `prototype/osfr-tools/tests/fishing_review.rs`: `b2ba0e7f...` -> `30b72712...`
  (the two finding probes updated to the fixed policy plus one regression).
- `prototype/osfr-tools/tests/fishing.rs` unchanged: `5d560b0d...`.

Full hashes are in `../lab/reports/fishing-service-repair-20261009/sha256-manifest.txt`.

## Service seam (for lead wiring)

`crate::fishing_service::FishingService` is `Clone`, bounded to
`MAX_SESSIONS` (4) tracked players, and explicitly ephemeral
(`SNAPSHOT_POLICY = SnapshotPolicy::Ephemeral`): nothing is persisted, and a
restart starts every player unregistered and idle.

- `new()` / `with_config(FishingConfig)`; `FishingConfig { bite_delay_ms,
  reel_window_ms, test_fish_id, test_fish_name, anchor }` is clamped into the
  declared min/max window by `bounded()`.
- `handle(&mut self, inner: &[u8], guid: u64, now_ms: u64) -> Result<Change, Error>`
  takes the opcode 138 payload exactly as `Gameplay::handle_in_world` sees it
  (the gateway already strips the outer tunnel) and returns inner packet bytes
  for the gateway to wrap.
- `tick(&mut self, guid: u64, now_ms: u64) -> Result<Change, Error>` makes the
  server-owned bite decision and closes a lost reel window. A tick for an
  untracked player is a no-op.
- `Change { replies: Vec<Vec<u8>>, changed: bool }`; `set_anchor(Vec4)`,
  `reset(guid)`, `reset_all()`, `phase(guid)`, `len()`, `is_empty()`.
- The caller owns the character position: no register request carries one, so
  the gateway should call `set_anchor(character.position)` before the first
  `handle` for that character.

## Wire schema provenance

Our client is `1.910.1.530630`. Every body below was re-read from the preserved
pseudocode (`../lab/decompiled/client/FreeRealms/pseudocode.c`) before the
pinned community forks were consulted; the PS3 and PC-2009 branches serialize
identical packet classes for all six classes compared (zero textual diff in the
six `Sanctuary.Packet/BaseFishingPacket` files inspected).

| Sub | Direction | Client reader | Verified on our client | Status |
| --- | --- | --- | --- | --- |
| 1 UpdateData | server | `FUN_00b66890` | handle + vector, 24 bytes | recovered |
| 2 RegisterPlayerRequest | client | `FUN_00b6a400` | handle + flag, 9-byte body | recovered |
| 3 RegisterPlayerResponse | server | `FUN_00b6acb0` -> `FUN_00b6a170` | player config 16 dwords + zone alias string + 5 dwords + int list + fish entry list | candidate (widths and order verified; values and int/float split are not) |
| 4 FishInfoUpdate | server | `FUN_00b6ad20` -> `FUN_00b691d0` / `FUN_00b673b0` | count + items; item = 3 dwords, 2 flags, dword, alias string, dword, flag, dword | candidate (same caveat) |
| 6 CastRequest | client | `FUN_00b6a460` | handle + vector + flag | recovered |
| 7 ReelInRequest | client | `FUN_00b6a4d0` | handle + flag | recovered |
| 8 SpawnBobber | server | `FUN_00b66a00` | handle + u32 + vector + vector, 44 bytes | recovered |
| 11 SpawnSchool | server | `FUN_00b69130` | id + vector + vector + list(3 dwords) + int list + 2 dwords, 68 bytes | candidate (same caveat) |
| 14 Result | server | `FUN_00b66be0` | recovered field order | recovered layout, candidate field meaning |
| 15 SpecialRequest | client | `FUN_00b6b510` | handle only, 8-byte body | recovered; divergence retained |

The sub 3/11 confirmation uses dispatch `FUN_00b6cd80` (cases at 1539238,
1539282, 1539673, 1539771) and the readers named above; line-by-line extracts
are in the report's `native-evidence.md`.

## Authority rules implemented

- **Registration.** A `RegisterPlayerRequest` binds the server-owned character
  GUID to the wire actor handle. A handle already bound to another GUID is
  refused (`ForeignActor`); a GUID already bound to another handle is refused by
  the tracker; a repeated registration from the same pair is a no-op with no
  reply. A new GUID at `MAX_SESSIONS` is refused (`AtCapacity`) instead of
  evicting a live session.
- **Cast (6) to bobber (8).** Only from the bound, registered session. The
  bobber carries the caster's own handle and a server-side code of 0; the
  experiment's random bobber GUID is not reproduced.
- **Bite.** The server owns the decision: `tick` moves a cast to the reel
  window only at or after `cast_at + bite_delay_ms`, never on a non-monotonic
  clock. No packet is emitted for the bite because no recovered fishing packet
  carries it; the client's own hook presentation and any tension/distance
  stream are still unrecovered.
- **Reel (7) and result (14).** A reel-in is accepted only in the reel window,
  and the result is emitted only from that accepted reel-in. The result is
  applied through `Session::server`, whose one-shot gate refuses a result that
  was not owned by a reel-in (R2) and refuses the same body a second time
  (R1: the wire carries no nonce, so the consumed landing is the only
  correlation). After landing the service re-arms the session to Idle so the
  next cast works without re-registering; the stale body stays refused.
- **Timers and bounds.** A lost reel window cancels the attempt with no result.
  Decode errors, oversized inputs (`MAX_BODY + 4`), truncation, trailing bytes,
  non-finite values and unknown sub-opcodes are refused without state change.
- **No reward path.** No coin, experience, item, quest or save mutation exists
  in the service. The result body is presentation only.

## Declared private reconstructed config

The pinned fork's registration values and its always-caught Trout are
experiments, not history. This task keeps exactly one declared private test
config, all in `FishingConfig`/constants so the lead can replace it:

- candidate registration values copied from the PC-2009 fork
  (`CANDIDATE_PLAYER_CONFIG`, `CANDIDATE_ZONE_CONFIG`, one fish entry with
  `CANDIDATE_FISH_TYPE` 418, two school offsets);
- `DEFAULT_BITE_DELAY_MS` 1500 (bounded 250..30000) and
  `DEFAULT_REEL_WINDOW_MS` 10000 (bounded 500..60000): no cast, bite or
  tension timing is recovered anywhere;
- result body: `CANDIDATE_RESULT_TYPE` 1, caught true, the configured fish id,
  and every unknown field zero.

The bite is not a fixed Trout award: which fish (and whether anything is
caught at all) stays a server decision parameter, and nothing is granted.

## Retained divergences and non-goals

- Sub 15 stays handle-only. The fork's SpecialRequest writes handle + 8 more
  bytes; our client's writer (`FUN_00b6b510`) emits the handle only, so the
  wider body is refused. The service sends no sub 16 response.
- Sub 18 SpawnFishRun, proxy broadcasts (sub 5/7 echoes), tension/distance
  streams and any underwater fish run stay unimplemented.
- The client's duplicate-cast guard remains the only directly observed
  ordering behavior; registration, hook and reel ordering are reconstructed.

## Verification

- RED: `scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml
  --test fishing_service` first failed with `couldn't find file
  tests\..\src\fishing_service.rs` (log `red-cargo-test-fishing-service.log`).
- RED for R1/R2: after the `Session::server` gate landed, the review harness
  failed exactly on its two falsifying probes, `replayed_result_...` (Landing
  re-applied) and `result_lands_while_idle_after_registration`
  (`OutOfOrder` unwrap), log `red-cargo-test-fishing-review.log`.
- GREEN: `--test fishing --test fishing_review --test fishing_service` passes
  **12 + 19 + 12 = 43 tests, 0 failed**, log
  `green-cargo-test-fishing-suite.log`. Baseline before this task was 12 + 18.
- `cargo check --lib` is clean apart from another worker's pre-existing
  `housing.rs` dead-code warning (`green-cargo-check-lib.log`).
- `rustfmt --edition 2024` on the four touched files exits 0.
- No native client run was made; the lead owns that gate. No captured fishing
  body exists in the lab, so the optional captured-body re-encode check could
  not be performed.

## Remaining proof needed before this is called complete fishing

1. A native capture of one registration and one cast from `1.910.1.530630` to
   confirm the sub 3/4/11 candidate values and the int/float split (the widths
   and order are already client-verified).
2. Evidence for when schools and fish info are (re)sent and whether the client
   requires the character-state flag (`0x400000`) the fork resends; that packet
   is not opcode 138 and stays out of this service.
3. The bite presentation: which packet or local timer shows the hook, and
   whether tension/distance meters need a server stream.
4. Result field semantics (`at_18` result type, `at_1c` caught, `at_20` fish id,
   `blob_2c` name) and whether the client accepts zeroed unknowns.
5. Any reward binding, quotas, rods, lures or deep-water rules; none are
   recovered, so no economy path was added.

## Skills applied

- `reverse-engineering`: reader-by-reader schema recovery in the preserved
  pseudocode before consulting community code.
- `game-research-websearch` / community-first order: the pinned Sanctuary PS3
  and PC-2009 forks were compared (`lab/references/Sanctuary-*`) and their
  divergence from our client recorded instead of transplanted.
- `game-recon`: kept fishing a native world activity, not a Flash minigame.
- `ponytail`: one service module, one test target, no new dependencies, no
  speculative abstractions, unknown bodies left unimplemented rather than
  fabricated.
- `RTK`: commands were read condensed; only the compile/test failures needed
  the full output.
- `caveman`: terse chat reporting; this note stays plain prose.
- `game-automation` / `asset-pipeline`: not applicable; no native run or asset
  work was performed by this task.
