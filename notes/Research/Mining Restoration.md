# Mining Restoration

MINING is minigame **#18 Mining, TYPE 8** in the recovered `MiniGameData.txt` inventory (`lab/reports/minigame-content-inventory.json`), with the regional variants **#91 Copper Mining**, **#92 Rich Copper Mining**, #93-100, and the duplicate #9999 all sharing TYPE 8. The movie is the original **MatchThree** family asset; the recovered `.lst` header names its own source path:

`C:\dev\FreeRealms\Live\Runtime\Client\Minigames\MatchThree\Mining\game_mining.swf`

## Recovered inputs (frozen originals, outside Git)

| Artifact | Pack / entry | CRC32 | SHA-256 |
| --- | --- | --- | --- |
| `game_mining.gfx` | `AssetsW_008.pack` 1399 | `df150264` | `b3c07e23eb32d16faa83f2c93f2e7a615be050493590248be9d03243f01b0908` |
| `mining_tutorial.gfx` | `AssetsW_008.pack` 1403 | `9a8e1aea` | `2e9ee186c7242a174d16f334fcf7d1f2d918968d8a67be93ae07f68c15d56f34` |
| `game_mining.lst` | `Assets_004.pack` 77 | `66499af9` | `cda2399ca8e170fd60c9182a19980d2327b155c95bda1692a8ab6170e62eadd1` |
| `mining_tutorial.lst` | `Assets_007.pack` 64 | `446f4c86` | `1836f206c8439e10606f5e3a0f76d1cea80a37bd558ba8ebbb3f3079f9f0fb39` |
| `MiningAssets.txt` | `Assets_005.pack` 3842 | `44da48c4` | `10e1e5e86a26b9f5a4cc0a0c0cee5a7a6586a1f94cd3ed4d1e2f1bad689ecf9b` |

Pack reader: pinned `dbg-pack` revision `8feac971cc21449b20f1cb58d0f1ff900fd6b849` (same revision as the earlier Cooking extraction). Movie decompiler: JPEXS 26.3.0, headless `-export script`, 106 ActionScript files from `game_mining.gfx` and none from the visual-only tutorial. All 129 `MiningAssets.txt` preload names match extracted catalog entries (`preload-verify.json`). Evidence: `../lab/reports/mining-restoration-20261009T015237Z/`.

## Shared TYPE 8 engine (original, byte-level)

`game_mining.gfx` and the already-restored Forging139 movie are the same engine: **78 of 106 scripts are byte-identical**, including `GameServer.as`, `GameServerNetwork.as`, `GameClientNetwork.as`, `GameLevel.as`, `GameData.as`, `core/ServerGrid.as`, `core/ServerState.as`, `core/ServerTileGenerator.as`, `MoldMeter.as` and `Progression.as`. Only config, sprite frames and a handful of tile/effect scripts differ.

Recovered protocol (original):

- Outer envelope, little endian: `[u16 39][sub u8][i32 instance][i32 -1][i32 gameId]`. Client `sub 5` RequestStartGame (GameId `-1`), `sub 6` Quit (`instance,-1,-1`), `sub 7` removal ACK, `sub 14` payload. Server `16` Info, `17` start ACK, `18` game over, `19` removal, `47` score rows; `26/12 MiniGameFlash` launches the movie.
- Payload `sub 14`: `i32 (len+1)` + tab-delimited ASCII + NUL. Compiled unchanged-community writer vectors reproduced by the pinned Sanctuary `BaseMiniGamePacket` oracle (native-proven `GameId=-1` sentinel); six vectors in `community-serializer-vectors.json`.
- Handshake: client `OnConnectMsg` -> `OnServerReadyMsg`; `OnStartGameMsg` -> `OnNextLevelMsg`; `OnStartLevelMsg` -> tile queue (board), build map, tile queue (70), random queue (70), `OnLevelInitMsg`, `OnQuotaInitMsg` per goal, `OnLevelEventMsg 32` meter init, `OnActiveMsg true`.
- Board: `aMyGrid` shape codes `0` empty, `1` regular random, `2` any random, `3` blocker, `4` dropzone, negative `-v` = specific value `v`, positive `v>=100` = category `floor(v/100)*100` with random value. Queue discipline: 50 minimum, 20 batch, 70 target; random queue `0..=999`.
- Chains: `OnChainMsg x,y,x,y,...`; at least 3, consecutive pairs orthogonally adjacent, no repeated coordinate, one selectable value (`400..499`), wildcard `200` matches and adopts the chain value, locked tiles unlock instead of exploding.
- Goals: collectable tiles (`600..699`) fall off the bottom into bins. `OnQuotaInitMsg(index,value,quotaTiles,perReward,iconId)`; `OnQuotaUpdateMsg(index,collected,quotaTiles,perReward)`; clusters are `floor(collected/perReward)`; the movie's own server-side check completes a goal at `collected >= quotaTiles`. `GameLevel.CheckCollectableQuota` is authoritative; reaching quota emits `OnStopLevel`; only the native `OnNextLevelMsg` yields the won state, and `FRServer_GameOver` emits `39/18`, `39/47`, `39/19`.

## Reconstructed (explicitly not original)

The MatchThree **server level tables** were never recovered (same gap documented for Forging139). Therefore: every concrete board layout, quota, spawn percentage, seed, RNG-to-tile mapping, instance-id namespace and reward/output mapping is local reconstruction, marked `reconstructed_rules: true` in content JSON and labelled in `src/mining.rs`. Mining's original per-node reward grant schema is likewise unrecovered, so the default `mining.json` proposes **no** transaction (`outputs: []`) until root supplies a recovered or explicitly accepted mapping.

## Rust artifact

`prototype/osfr-tools/src/mining.rs` — data-driven `Type8Engine` (std + serde only, self-contained), shared by Mining18 and, by data, by the other TYPE 8 games:

- `Type8BoardSpec`, `Type8Goal`, `Type8Output`, `Type8Kind`, `Type8State`, `Type8Run`, `Type8Phase`, `Type8Transaction`, `Type8Outcome`, `payload_packet`.
- `Type8Engine::{load, state, active, spec, validate_state, begin, cancel, tick, handle}`; `mining_spec()` gives the reconstructed Mining18 default.
- Content file wrapper: `{schema_version, game_id, profile_id, name_id, description_id, icon_id, spec}`; `load` validates the game binding, derives and validates the playable rectangle and dropzone row from `shape`, and enforces the instance namespace.

This interface was published to the Harvesting111 worker and root at first recovery, before implementation, so harvesting can emit byte-compatible `Type8BoardSpec` JSON (`content_file = "harvesting"`, ids 58/111/...) without editing shared files. Mining owns the engine; harvesting owns its config/profile/result data.

## Tests (owned, no `lib.rs` edit)

`prototype/osfr-tools/tests/mining.rs` path-includes `../src/mining.rs`. RED first (module absent -> `couldn't find file tests\..\src\mining.rs`, retained in `focused-red.log`), then GREEN:

`cargo test --test mining` -> **17 passed, 0 failed, 0 ignored** (`focused-green.log`). Coverage: pinned payload/oracle framing, info packet + idle-only begin, full handshake message set, chain validity (minimum, orthogonality, repeats, colour, idle, bounds), wildcard and all-wildcard chains, collectable fall progress with multi-goal completion, invalid fall rejection, win transaction once-only + monotonic replay, authoritative timer loss, quit-only-before-win, serde save/restart resume, tamper rejection, verify reports, game-namespace binding, default-spec consistency.

## Skills applied (and what they changed)

- `mod-any-game` + `reverse-engineering`: community reader/decompiler before any custom format work; read the movie as the spec; oracle-style evidence instead of guessing.
- `game-recon` / `game-research-websearch`: pinned community sources first (`dbg-pack`, Sanctuary, Legacy) and the verified wiki corpus at `lab/references/freerealms-wiki/wiki.sqlite` (read-only; Copper Mining page is a stub, Mini-Games page adds no protocol).
- RTK: normal commands first; evidence text kept in task-owned files.
- Ponytail: one self-contained file, no new dependencies, no generic trait/factory, no speculative powerup engine; deleted-work warnings fixed rather than papered over; one runnable focused check suite instead of broad scaffolding.
- Caveman: final report terse, payload verbatim.

## Module integration contract (for root)

1. Register `pub mod mining;` in `lib.rs` and add mining state to the gameplay/gateway snapshot as a new optional field (serde default) so older saves still load.
2. Ship a `mining.json` next to the campaign content. Ship the reconstructed `mining_spec()` data until original level tables exist, or a Copper Mining variant with `game_id: 91`.
3. Call `begin(profile)` only after the accepted Miner job/proximity/content checks (the opening content binds Miner to `required_profile: 14`), stage rewards through `Type8Outcome ::transaction` on the gameplay clone, persist the snapshot, then send replies. `cancel()` on reconnect; `tick(elapsed)` for the authoritative timer.

## Remaining native gates

No native acceptance has been attempted. Open: real client entry/tutorial, drag chains against the Rust mirror, queue/gravity without desync, ore bins and cluster display, `OnStopLevel`/`OnNextLevelMsg`/`OnGameWonMsg` timing, `FRServer_GameOver` results close, restart/replay, and the actual item grant once a mapping is recovered or accepted. The original Mining level/quota tables and reward schema remain unrecovered.
