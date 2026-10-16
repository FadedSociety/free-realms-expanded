# Delegated fishing readiness

2026-10-09. **Finding: fishing cannot start. The delivered wire module is correct but unwired, and the server side of the whole cast-to-catch loop is missing.** `src/fishing.rs` and its tests compile only through `#[path]` includes; nothing in `lib.rs`, `gameplay.rs` or `gateway.rs` names fishing or opcode 138, no hook/bite decision exists anywhere in the repository, and no code sends the registration response, school spawns or fish info the native client needs before the fishing activity can run. The prior note's claim that community repositories define nothing but names is superseded: the pinned edenfps Sanctuary branches contain full fishing serializers, handlers and session state, with a version caveat and one observed width divergence.

Pinned references: Sanctuary PS3 `d1c747f64653264cd52b854efd6c17de6d418a78` (branch `ps3`) and Sanctuary PC-2009 `b1182d671d8fcc1e44efa1909e6b3cc5056df8e6` (`HEAD`), both under `lab/references/`, as inventoried by [[Edenfps Toolkit and Sanctuary Branch Review]].

## Verified current state

- `src/lib.rs` lists 34 modules and none is `fishing` or `restoration`. A tree-wide search for `fishing` outside `src/fishing.rs` matches only the catalogue file `src/restoration/piece_018.rs`, the stat-name table in `piece_047.rs`, and unrelated strings; `gameplay.rs`/`gateway.rs` have no fishing or opcode-138 dispatch.
- The catalogue entry (`src/restoration/piece_018.rs`, around line 208) lists fishing as `Status::CodeTestCandidate` with `Entry::ModuleDriver` api `fishing::Session::request(ClientRequest)` and driver `cargo test --test fishing`. That file is not compiled into the crate, and the Python audit `tests/pieces/test_piece_018.py` only checks catalogue text, module path and test path; it does not prove the module is reachable from the server.
- Module and tests are unchanged from the builder report: `src/fishing.rs` `eee55aeda...`, `tests/fishing.rs` `5d560b0d...`; the independent review harness `tests/fishing_review.rs` remains `b2ba0e7f...`.
- Reviewer reruns today: `scripts/cargo.ps1 test --test fishing` -> 12 passed, 0 failed; `--test fishing_review` -> 18 passed, 0 failed; no warnings. Log: `lab/reports/delegated-fishing-audit-20261009/cargo-test-fishing-and-review.log`.

## Exact missing gates

1. **Registration/entry.** Nothing answers sub 2. To bring the native fishing state up the server must send sub 3 (register response: player config + zone config + fish model ids + fish entries), sub 1 (update data), sub 4 (fish info) and sub 11 (school spawns). Our module decodes sub 1 and passes 3/4/11/16/18 through as opaque bytes.
2. **Cast and bobber.** Sub 6 decodes; nothing spawns a bobber (sub 8). The community experiment invents a random bobber guid.
3. **Hook/bite: no authority exists.** No timers, thresholds, tides or bite decisions anywhere; `Session::hooked()` has no caller. The native tension/distance meters are driven by server sub 1/4 streams we cannot yet produce.
4. **Reel and result.** Sub 7 decodes and sub 14 decodes with validation, but nothing decides a catch, binds a fish id, or grants anything. The independent review's R1 (replayed result re-applies) and R2 (result lands from Idle) must be closed inside `Session::server` before any reward is attached.
5. **Lifecycle wiring.** `Session::request`/`Session::server` are not driven by the gateway; opcode 138 is not routed, so no fishing packet reaches the module in a running server.
6. **Sub 15 width divergence.** Our client's writer `FUN_00b6b510` emits handle-only (8-byte body, 12-byte packet; our module matches and the native writer was re-verified in the prior review). PC-2009 Sanctuary serializes `Guid + ulong Data` (16-byte body). Our module follows our client; Sanctuary's special-request handler would not read our client's packet, so that layout must not be transplanted.

## Corrected community evidence

The old Universal Modder assessment said the community defined only fishing names. That is now wrong for these pinned forks.

- Agreement with our module (byte-compatible serializers): envelope `BaseFishingPacket` writes opcode 138 then a `short` sub-opcode; sub 2 `Guid + byte`; sub 5 `Guid + int + Vector4` (our "distance" f32 is their unknown int; same 4 bytes, native NaN check supports float); sub 6 `Guid + Vector4 + bool`; sub 7 `Guid + bool`; sub 8 `Guid + int + Vector4 + Vector4`; sub 10 `Guid + int + byte + byte`; sub 12 `int`; sub 13 `int + Vector4`; sub 14 field count and order. `PacketWriter` frames strings as `u32 byte length + UTF-8` and lists as `u32 count + items`, matching our count-prefixed blobs.
- New candidate bodies not yet capture-confirmed for our client: sub 3 register response (player config of 16 ints/floats, zone config string + 5 values, int list, fish-entry list); sub 4 fish info (list of `ClientFishEntryInfo`: 3 ints, 2 bools, int, string, int, bool, int); sub 11 spawn school (int + Vector4 + Vector4 + fish list of 3-int items + model-id list + 2 ints); sub 16 special response (Guid + int + bool); sub 18 spawn fish run (bool + int + string + spawn list). The sub 11 prefix (id + two vectors) is consistent with our native reader `FUN_00b69130`.
- Result semantics candidate: `ResultType`, `Caught`, `FishId`, `FishName`, then seven unknown ints, two unknown strings, then int/int/bool/int. That maps one-for-one onto our decoded layout (at_18, at_1c, at_20, at_24, blob_2c, the seven middle dwords, blobs 58/64, at_74/at_7c/at_80/at_78) including the odd trailing order. Treat as serializer-derived semantics, not history.
- Experiment behavior to keep out of our authority: the PC-2009 reel-in handler always returns a caught Trout (`FishId = 418`), registration sends two fixed sample schools beside the player, and the session/zone helpers are partly dead code. No persistence, no reward, no historical balance.
- Version caveat carried from the Edenfps review: both branches target PS3 and PC-2009, not our Windows `1.910.1.530630` client; the sub 15 divergence is a concrete example. Nothing here should be transplanted without capture parity.

## Minimum future native acceptance

1. Lead-owned integration: `pub mod fishing;` and gateway routing of opcode 138 payloads to `ClientRequest::decode` -> `Session::request`.
2. Native capture of one fishing attempt (cast -> bobber -> hook -> reel) to pin sub 3/4/11/16/18 bodies and confirm sub 15 width for our client.
3. Implement the registration response, school spawn and fish info so the fishing activity starts in the native client (the fork uses character state `0x400000`; verify against our client).
4. Close R1/R2, then add one explicit, documented server-side bite decision and emit the result only after a real reel-in.
5. Native run proving cast -> hook -> reel -> result, plus the negative: no result without a reel-in and no reward until economy integration is separately authorized.

Reviewer changed no shared file, UI, save, release artifact or native state; the Sanctuary forks were read, not built or executed. Receipt: `lab/reports/delegated-fishing-audit-20261009/receipt.json`.
