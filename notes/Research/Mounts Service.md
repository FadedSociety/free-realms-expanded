---
type: research
updated: 2026-10-13
---

# Mounts service (W25)

Prototype service and content slice. It is not registered in `lib.rs` or `gateway.rs`, and it has no native acceptance capture.

## Recovered client facts

- The shipped client resource `lab/runs/client-baseline-20261010T064034Z/client/Resources/AnimationTypes.xml` has 30 `MountType` elements: empty id `0` plus 29 positive animation types. SHA-256: `20c55ba0a78171635431cf2d2e84c3d342812eda6079777b41db6af640c90441`. Id `0` is retained as a sentinel; ids `1..=29` are the usable animation prefixes. The table does not map individual ride IDs to animation types.
- Mount family is `0xa8`/168. The client reads a `u16` family followed by a `u8` sub-opcode (`00b03710`, `00b04c20`). The raw datagram framing and byte order have not been captured.
- `Mount.Spawn` (`00bff6f0`) checks the shared request window (`00a2e540`), records the timer, sends dismount sub-opcode `3`, sends spawn sub-opcode `6` with an `i32` mount id, and sets the automount id. This sequence is supported by disassembly at `00bff6f0` and the packet constructor at `009aa410`.
- Lua `Mount.Dismount` (`00bffa60`) and `Mount.Despawn` (`00bff8e0`) both use the same dismount request constructor (`009aa410`, family 168/sub 3). They differ in their dock callback values. The client cannot tell the server which Lua function ran.
- The mount list record reader (`008e74c0`) reads mount id, name string id, image set id, actor GUID, members-only byte, tint, tint-alias string, upgradable byte and upgraded byte. Packet sub-opcode `5` replaces the list (`00b06100`, list reader `00b05e00`). The login self body also includes the list after pets (`00944fe0`, see the Mounts Build Guide).
- Mount response sub-opcode `2` reads two GUIDs, seat and result integers, two further integers and a float (`00b04c80`). Dismount response sub-opcode `4` reads a GUID and integer (`00b04dc0`). Spawn sub-opcode `6` writes one integer (`00b05a80`); item spawn sub-opcode `8` writes an integer and byte (`00b05ae0`).
- The client sends mount request sub-opcode `1` when the spawned ride actor appears and its GUID matches the automount record (`0096de80`, request sender `00b04290`). The actor/list order is inferred from pseudocode, not a captured server exchange.
- The client sends dismount requests on Flash minigame start (`009bd650`, `009beb70`) and before some ability starts (`00a35600`). The client blocks relevant repeated requests for about one second (`00a2e540`).
- The client does not expose a ride speed value in the reviewed tables. The Build Guide's 150% speed figure is wiki-only. No mount traffic was captured.

## Data provenance and limits

`prototype/osfr-tools/data/content/mounts.json` combines two distinct sources:

- `mount_types`: copied from the client `AnimationTypes.xml` table above. Each row has a `client` source tag and the input file hash.
- `mounts`: 54 data rows seeded from `lab/references/FreeRealms-Legacy/src/Resources/Mounts.json` (SHA-256 `e066e0a37e7fb389d05d600d7b153b3c0d8880c33878d757b7541d97953f840f`). `ClientItemDefinitions.json` SHA-256 is `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`. The pinned repository revision is `9e082025391177d5a33ee18c714ddd06b457a695`; the repository is AGPL-3.0. No source code was copied. System ride names, IDs, text IDs, image set IDs, model IDs, upgrade flags, tint data and `Stats.MaxMovementSpeed` retain Legacy source tags.
- Candidate inventory item definitions are the Legacy `ClientItemDefinitions.json` rows whose `NameId` and `Comment` match a mount row. This yields 210 candidate item IDs across 54 rides. Name IDs are one-to-many, so the data retains candidate sets instead of choosing one row. The mapping is tagged reconstructed: the client item-use path proves that item definitions can request a ride (`00b06db0`/168/8), but the client resource does not identify which item definitions belong to which mount IDs. The service currently treats a candidate item definition present in inventory as ownership (R); validate this crosswalk before promotion.
- `mount_type_id` remains null for every ride because the client table maps animation prefixes, not ride IDs. `members_only=false`, `is_upgraded=false`, and `seat_count=1` are slice defaults (R), not recovered per-ride values.
- `max_movement_speed` retains the Legacy value, including zero for the Lawn Chair. The service exposes it to world spawn effects as reconstructed movement input (R); its units and client parity are unknown.

## Service behavior

`src/mounts.rs` is session-scoped. It derives the owned roster from positive-count inventory items and the content crosswalk. Spawn and mount each check ownership; spawn and mount reject minigame, active-ability and zone restrictions (R). Spawn requests use a one-second server duplicate guard (R), separate from the client's recovered guard.

Spawn returns a synthetic 168/5 list update containing the new actor GUID before the world-actor spawn effect, so the client can match automount when the actor appears. The integration layer supplies the actor GUID and routes the world effect through the actor/interest system. Mount returns synthetic 168/2 bytes to the owner and supplied nearby observers. Dismount returns 168/4 to those recipients and leaves the ride actor spawned. A later identical 168/3 request, when the player is no longer mounted, removes that actor and sends a 168/5 list with an empty GUID (R disambiguation). Minigame and ability hooks call the same dismount path.

Packet tests assert little-endian synthetic inner bytes, following this crate's serializer convention. This does not establish captured mount-wire byte order or prove that the selected default fields match the original server. No gateway tunnel wrapper is emitted here.

Active actor GUID and rider state remain session-only. No save schema change was made: client evidence requires an owned list at login but does not show that an active mount must survive logout/login.

## Verification

- Native-map checks: `python -I scripts/native_map/nmq.py dis 00a2e540 192` confirmed the elapsed timer guard's 1000-unit threshold. Disassembly checks also covered `00bff6f0`, `00bff8e0`, `00bffa60`, `00973050`, `00b04c20`, `00b04c80`, `00b04dc0`, `00b05a80`, `00b05ae0`, `00b05e00`, `008e74c0` and `00b06100`. Unknown labels used by the implementation were checked against bytes.
- Resource/data assembly: isolated Python (`python -I -`) parsed the hashed client XML and the pinned Legacy JSON inputs, tagged each field, checked 30 client type rows/29 usable types and 54 mount rows, and wrote `data/content/mounts.json`.
- Formatting: direct private toolchain `rustfmt.exe` on `src/mounts.rs` and `tests/mounts.rs`. The cargo-bin rustup shim failed because no default toolchain was configured; direct pinned `stable-x86_64-pc-windows-gnu` formatting succeeded.
- First test run rejected the Legacy Lawn Chair's `MaxMovementSpeed=0.0`; zero is valid for that stationary ride, so validation now rejects only negative or non-finite speed. Final test command from `prototype/osfr-tools`: `$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W25'; & ..\..\scripts\cargo.ps1 test --test mounts` — 9 passed, 0 failed. The crate emitted one existing dead-code warning in `src/housing.rs:567`.

## Integration and open gaps

- Lead adds the module and routes 168/1, 3, 6, 8 and 14 in `lib.rs`/`gateway.rs`; those shared files were intentionally untouched. Route typed inventory and context into each per-session `MountService`.
- Route `SpawnRide`/`DespawnRide` through world actor creation and `Players` interest fanout. Late joiners need the client pending-mount values in the player update; their subtype and five fields remain unresolved. The service's 168/2 and 168/4 observer fanout is a reconstructed choice.
- Add the mount list to the self packet after pets and send list updates through the normal tunnel envelope. The service does not serialize full actor spawns or tunnel wrappers.
- This slice's inventory-derived ownership follows W25, while the guide describes a separate persistent mount roster plus an item-triggered spawn path. The 54 Legacy rows are only a seed, not the guide's full 113-ride wiki roster. Resolve the ownership model and expand/validate content before promotion.
- Trials, trial expiry/refusal, upgrade item consumption and response, membership enforcement, ride gliding, movement authority, mounted restrictions and active actor persistence are not implemented. The guide does not establish active-mount persistence across logins.
- Service refusals return typed errors; the gateway must turn them into visible client effects before integration is considered complete.
- No server writer, packet byte order, datagram capture, actor seat definition mapping, original server rules or native visuals were verified. Current tests establish the local reducer and synthetic packet bytes only.
