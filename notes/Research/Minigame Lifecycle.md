# Minigame Lifecycle

Snapshot: 2026-10-13. W15 adds an isolated lifecycle ledger and protocol tests. It is not registered in `lib.rs`, adapted into an activity, save-integrated, or natively accepted.

Workflow: Ponytail reused `rewards::apply_grant` and added no dependencies; Caveman kept the evidence note concise; reverse-engineering was applied through read-only `nmq dis` checks. Game recon, web research and asset-pipeline work were not needed for this lifecycle-only slice; the native client was not launched.

## Evidence and packet contract

Primary source is `notes/Build Guides/Gameplay/Minigame Framework.md`, narrative and Network sections, reviewed against the native map and accepted Cooking, Smelting, Forging and encounter runs. This note does not claim original-server traffic; the guide records no original-server captures.

| Direction / transition | Recovered client contract | Provenance |
|---|---|---|
| S>C offer, `39/16` | Header is `u16 39, u8 16, i32 instance, i32 -1, i32 game`; then six metadata dwords, member byte, three reward bundles, counted `0xe8` goal records, five flags, preload string and tail fields. The `+0x268` flag must be set to open results. | Reader `009bfc50` / `009bdd70`; state creation `009bfdb0`, `009beb70`, info copy `00c42c60`. Accepted activity writers in `cooking.rs`, `smelting.rs`, `forging.rs`; guide “Info record carried by 39/16”. |
| C>S start, `39/5` | Header identities are `[instance, -1, -1]`; the last value is a sentinel, not the authored game id. Ordinary server response is `39/17`, then `26/12` for Flash games. | Request `009be7d0`, writer `009bdc60`; client start reader `009b9550`; Flash handler `00aa0a10` -> `009bd650`; native Cooking/Smelting/Forging receipts. |
| Encounter start | Framework guide distinguishes encounter start `41/2` from ordinary `39/17`. The encounter packet has a 12-byte header, player GUID and trailing zero byte. | Framework dispatch `00aa36c0`, body reader `00a9f760`; packet parser `008d6690`; body layout is also recorded in `src/restoration/piece_060.rs`. The guide says encounter start activates the minigame state without ordinary `39/17`. |
| C>S / S>C movie relay, `39/14` | Payload is a length-prefixed string including its NUL terminator. Movie handshake and game messages remain activity-owned. | Client send path `009be6b0`; client receive path `009b89e0`; native Cooking, Smelting and Mining packet fixtures. |
| S>C win/loss | Movie event (`OnGameWonMsg` or `OnGameLostMsg`) precedes `39/18`, `39/47`, `39/19`. `39/18` carries won byte, stars, elapsed seconds and score. `39/47` carries at least one actual `scoreTotalScore` row (type 4). | Result reader `009ad480`; score dispatcher/row reader `009bb030` / `009b2b30`; removal handler `009bf190`; native craft and cave result runs. |
| C>S result acknowledgement, `39/7` | Automatic acknowledgement after `39/19`; it is not the player's results Close. Keep the Won/Lost run until Close. | Client writer `009be8f0`; native Cooking, Smelting and Forging receipts. |
| C>S leave, `39/6` | Ordinary pre-win quit uses `[instance, -1, -1]`; server answers with `39/19` only. Do not send `39/18` or `39/47`. | Quit path `00c17470` -> `00939100`; removal handler `009bf190`; accepted native Cooking Playing Quit. |
| Results Close | Client sends four-byte `26/42`; encounter Close also sends `41/109` with header identities `[0, instance]`. | Close writer `0092f300`; leave path `00c172c0` -> `008d2730` -> `009bd240`; encounter exit reader in `src/encounter.rs` validates a 12-byte packet. |
| Prize wheel | Server selects the item and sends `39/45` before the spin. Client's `39/46` only reports that the animation stopped; it cannot choose the prize. | Wheel packet/handler `009b6da0`; constructor `009aa0c0`, sender wrapper `009bda80`; guide “Prize wheel”. |
| Disconnect | Movie `GameClientDisconnect` is empty; there is no client resume/refund path. | Exported movie `SoeGameFramework.as`; guide “Leave, quit, cancel and disconnect”. |

Family 39 is bidirectional and much larger than lifecycle controls. The framework dispatch table also covers info, Flash relay, round/status, group, score, objective, wheel and reward-preview sub-opcodes. W15 consumes C>S start `39/5`, leave `39/6`, acknowledgement `39/7`, wheel stop report `39/46`, result Close `26/42`, and encounter exit `41/109`. It emits `39/16`, start acknowledgement, movie terminal event, result/score/removal packets, Flash launch `26/12`, and a server-selected `39/45` landing entry. It does not consume activity-specific `39/14` commands or implement reward-preview `39/68` updates.

`python -I scripts/native_map/nmq.py dis 00aa36c0 512` showed the encounter dispatcher calling the 12-byte encounter-header reader and dispatching case 2 to `ParseEncounterPlayerEnterBody` at `00a9f760`. Separate `nmq dis` checks of `009b9550`, `009bf190`, `009b2b30` and `009ad480` were consistent with the guide's start, removal, score-row and game-over handlers. No speculative/unknown label is used as authority for payout or activity rules.

## Reconstructed server policy (R)

- One active run per player; mint positive instance ids from a persisted monotonic counter. The original server's instance scheme is unknown.
- Treat `Activity::score` and `Activity::finish` as server authority. Client-reported win/score text does not settle a run.
- Leave and disconnect before a win cancel without reward or material consumption. Activity materials remain untouched by this ledger; adapters stage consumption only with a successful win transaction.
- Keep Won/Lost until a matching Close. Duplicate finish, acknowledgement and Close events have no second reward or packet effect.
- `minigame-run:<instance_id>` is the stable grant key passed to `rewards::apply_grant`. Prefixing is a reconstructed namespace choice; the numeric run id makes retries stable.
- `RunSpec::empty` uses empty 69-byte bundles, no goal rows, ordinary difficulty 1, all five info flags set, and the accepted activity writers' remaining tail defaults. These generic content values are not original game facts.
- The lifecycle returns reward messages separately from the exact framework result sequence. The caller must commit the returned ledger and `RewardState` together with activity/material state before sending either set. The reward API itself stages inventory, profiles and grant keys; it does not write a save.

## W15 implementation and checks

`prototype/osfr-tools/src/minigame.rs` provides:

- persisted `MinigameRuns` with a checked counter, one current run and one last terminal record;
- `Offered -> Starting -> Playing -> Won/Lost -> Closed`, plus pre-win `Cancelled` paths;
- exact family-39 headers and message bodies, ordinary Flash start, encounter `41/2`, leave, terminal results, and close parsing;
- `39/45` server-selected wheel entry and an inert, once-only `39/46` stop report;
- an `Activity` seam for score and terminal outcome; no activity module implements it yet;
- atomic reward staging through `apply_grant` and the run-derived grant key.

`prototype/osfr-tools/tests/minigame.rs` is standalone via `#[path]`, like `tests/rewards.rs`; no shared module registration changed.

Commands and results:

- `python -I scripts/native_map/nmq.py dis 00aa36c0 512` — dispatcher/header/body path inspected.
- `python -I scripts/native_map/nmq.py dis 009b9550; python -I scripts/native_map/nmq.py dis 009bf190; python -I scripts/native_map/nmq.py dis 009b2b30; python -I scripts/native_map/nmq.py dis 009ad480` — lifecycle readers inspected.
- `$env:CARGO_TARGET_DIR='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/build/W15'; ./scripts/cargo.ps1 test --test minigame` from the repository root — failed before compilation because the Cargo manifest is nested under `prototype/osfr-tools`.
- The same focused test command first failed after adding persisted wheel rows because `RewardBundleRow` has no serde derives. W15 stores the wire row in a small serializable `WheelEntry`; the final focused run below passes.
- `$env:CARGO_TARGET_DIR='%USERPROFILE%/Documents/Projects/CartoonMMO/lab/build/W15'; ../../scripts/cargo.ps1 test --test minigame` from `project/prototype/osfr-tools` — **7 passed, 0 failed**. The library build reported one pre-existing unused-method warning in `src/housing.rs::Lot::instance_index`; W15 introduced no warning.
- `& '../lab/tools/rustup/toolchains/stable-x86_64-pc-windows-gnu/bin/rustfmt.exe' 'prototype/osfr-tools/src/minigame.rs' 'prototype/osfr-tools/tests/minigame.rs'` from `project` — formatted only the two owned Rust files. The initial rustup-proxy `rustfmt.exe` call failed because this private toolchain has no configured default; the direct installed toolchain binary succeeded.

## Activity migration plan

Keep each activity's board/movie parser, content validation, authoritative goals, score calculation and reward transaction. Replace only its duplicated shared lifecycle boundary with `MinigameRuns`/`Activity`; serialize through a cloned outer gameplay snapshot, save all state atomically, then send returned packets.

1. **Cooking** (`src/cooking.rs`): adapt `Cooking::begin` and lifecycle branches in `Cooking::handle` (39/5, 39/6, 39/7, `FRServer_GameWon/Lost/GameClose`); replace `info_packet`, `header`, `game_over`, `results`, `launch_packet`. Preserve six-phase validation, recipe `CookingTransaction`, and accepted row-46 receipts in [[Native Cooking Finish and Mae 171251]] and [[Ordinary Crafting Quit Acceptance]].
2. **Smelting** (`src/smelting.rs`): adapt `Smelting::begin` and lifecycle branches in `Smelting::handle`; replace matching info/header/result/launch writers. Preserve four-round checks, recipe transaction, and accepted row-77 receipt in [[Native Gear Up and Smelting 173814]].
3. **Forging** (`src/forging.rs`): adapt `Forging::begin`, start/quit/ack and terminal branches in `Forging::handle`; replace its info/header/result/launch writers. Preserve board simulation, pause/timeout checks, and recipe transaction; retain row-139 win and cancellation restart receipts in [[Native Forging Victory 181715]] and [[Native Crafting Quit 005748]].
4. **Mining** (`src/mining.rs`): adapt `Type8Engine::begin`, `cancel` and the lifecycle controls in `Type8Engine::handle`; replace `info_packet`, family header, result, launch and close handling. Keep the Mining board, queue, score/quota authority and `reward_transaction` activity-owned. Its native partial board/quit receipt is not a win/payout gate.
5. **Opening** (`src/opening.rs`): no family-39 lifecycle was found; no adapter migration is indicated.

## Remaining gates

- Registering the module, save-schema integration, save-failure rollback and real reward commit/send ordering are lead-owned integration work. `Reward Transactions.md` records the applied-key save dependency.
- No activity adapter or native client test is part of W15. Preserve all three accepted Cooking/Smelting/Forging receipts during integration; do not claim generic playable minigames.
- The reviewed guide describes encounter `41/2` as the start trigger without ordinary `39/17`. Existing `src/encounter.rs::on_zone_ready` emits both `41/2` and `39/17` in its local accepted flow. W15 follows the guide branch and does not edit that shared module; reconcile this difference against the encounter native receipt before replacing its sequence.
- Original reward amounts, star formulas, wheel odds/order, objective content and original server packet timing remain unknown. The wheel entry body and its identity match against the 39/16 preview remain activity-supplied and unverified; `39/68` reward preview updates remain outside this module.
- Activity methods and transaction state are not save-coupled here. A failed outer save must discard the staged ledger, reward and activity snapshots before any reply is sent.
