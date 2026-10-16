# 009 Checkers

Work item 009 acceptance: recover the checkers rules/config, including
compulsory captures, promotion, end state and replay protection. Status:
**code + focused tests candidate; no native or integration acceptance yet.**
Nothing here grants currency, items or minigame rewards, and no shared
integration file was changed.

## Recovered inputs (frozen originals, outside Git)

| Artifact | Location | CRC32 | SHA-256 |
| --- | --- | --- | --- |
| `MiniGameData.txt` | `lab/working/OSFR.Client/OSFR Client/Resources` | - | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `MiniGameTypeData.txt` | same directory | - | `2d200849d113c0f4b071b869774843b2fe9d07095235e7a6e907be83156e27d0` |
| `MiniGameGroupData.txt` | same directory | - | `3b288fa6e665c5ea41122195273ae846251835c8fc323183407e0d48ca969048` |
| `MiniGameGroupLinks.txt` | same directory | - | `bff472fb91e88ebf6077f541012701cc774dcfb1d7b89261b2ad92d95ffcb623` |
| `game_checkers.gfx` | `AssetsW_010.pack` 853 -> `pack_10/000853.gfx` | `3145712d` | `c7c54c5a1013e0346779ec9dd0bd90996270236a799dc75e6d57008f29a7eb27` |
| `tutorial_checkers.gfx` | `AssetsW_010.pack` 851 -> `pack_10/000851.gfx` | `f9d0a965` | `15a59967e3146f02b97abd461f98d90406e13bd5e47f830f6792f61ebce3a328` |
| `game_checkers.lst` | `Assets_000.pack` 1311 -> `pack_12/001311.lst` | `a25934da` | `a70af4a95ee6bce97d7cb56eaf20884b50393d4fce961594ba8a40f70870b155` |
| `tutorial_checkers.lst` | `Assets_001.pack` 5 -> `pack_13/000005.lst` | `c2c8959e` | `704163a7e6cdb697baa33482c13f5499c572563a7f9e266c7a8accdc4cb9c07b` |
| `icon_category_minigames_checkers_64.dds` | `AssetsW_004.pack` 3202 | `74deb1cf` | recorded in the evidence config |

Both `.lst` headers name their original build path, which pins the movie as the
source of the rules:

```
#CRC64=0xda08f5d2faa55687,srcfilename=C:\dev\FreeRealms\Live\Runtime\Client\Minigames\Checkers\game_checkers.swf
#CRC64=0x2e6a74690b0f070a,srcfilename=C:\dev\FreeRealms\Live\Runtime\Client\Minigames\Checkers\tutorial_checkers.swf
```

Movie decompiler: JPEXS FFDec 26.3.0 (pinned under `lab/tools/ffdec-26.3.0`),
headless `-export script`. The game movie yields 55 ActionScript files; the
tutorial movie yields two trivial scripts (`stop()`,
`Object.registerClass("Checkers",PieceView)`) and contains no rules.

## Catalogue facts (original)

`MiniGameTypeData` row 7: `IS_FLASH=0`, `IS_MICRO=0`, `IS_ACTIVE=1`,
`NAME_ID=0`, `DESCRIPTION_ID=0`, `HOWTO_SWF_NAME` empty.

`MiniGameData` TYPE 7 rows (all six): ids `21, 68, 69, 70, 541, 542`,
`NAME_ID=900`, `DESCRIPTION_ID=901`, `ICON_ID=917`, `DIFFICULTY=2`,
`MEMBERS_ONLY=0`, `MIN_PLAYERS=0`, `MAX_PLAYERS=0`, `PRELOAD_FILE` empty,
`LEADERBOARD_ID=0`. Locale text: 900 `Checkers`; 901 `Enjoy all of the classic
fun of Checkers within the world of Free Realms!`.

`MiniGameGroupLinks` rows 212-217 map group 7 to those six games. Group 7's own
`MiniGameGroupData` row carries only zeroes, so the group has no independent
name.

Two source quirks are recorded rather than repaired: `ICON_ID=917` resolves to
an unrelated item string (`Elite Pink First Aid Pants`), and the declared
`IS_FLASH=0` disagrees with the shipped movie. The checkers category art is the
separate `icon_category_minigames_checkers_64.dds` asset.

## Recovered rules (original, from the movie ActionScript)

`BoardProxy.as` (`ConfigServer.as` supplies `GAME_ID = 21`):

- 32 playable spots on an 8x8 board, 12 pieces per side, 24 pieces total
  (`SPOTS`, `PIECES_PER_SIDE`, `PIECES`).
- `ProxyReset` places the non-red side on spots `0..=11` and the red side on
  `20..=31`, all pawns. Network spots are canonical: red occupies `0..=11` and
  walks towards row 7; black occupies `20..=31` and walks towards row 0.
  `SpotToNetwork`/`SpotFromNetwork` mirror the board (`31 - spot`) for the red
  client.
- `GetSpotFromOffset(spot, dx, dy)` defines the geometry: `row = floor(spot/4)`,
  `col = spot % 4`, odd rows shifted left one column slot, so `dx` 0 and 1 are
  the two real diagonals. Pawns keep the forward pair; kings keep all four
  (`GetMoves`/`GetJumps`).
- A capture jumps an adjacent enemy onto the empty square beyond it
  (`CanJump`: target must be the other colour and the landing square empty).
- Captures are compulsory. `StartMove` first offers every friendly piece
  `CanMove(piece, true)` (jumps only) and only falls back to `GetMoves` when no
  jump exists; the UI shows `ForceJump` in that pass.
- After a jump the turn is locked to the jumping piece (`mJumpPiece`):
  `CanMove` returns only that piece's jumps until none remain, so one submitted
  move is one complete capture chain.

`GameServer.as`:

- `IsJump(spot, dest)` computes the captured spot from the row parity
  (`(spot + dest + 1) / 2` on even rows, `(spot + dest - 1) / 2` on odd rows)
  and returns `EMPTY_SPOT = -1` for a one-row step.
- `IsKing(dest)` promotes on row 0 or row 7. Promotion applies inside a chain,
  so a promoting jump continues with king directions. The check is
  destination-only, so a king that steps back onto row 0 or 7 re-emits
  `MOVE_KING`; the client treats that as an idempotent `Kinged` and replays the
  king sound.
- `OnMove(start, moves)` splits the comma-separated destination list and emits
  per hop `MOVE(10)` plus `MOVE_JUMPED(11)` for the captured spot and
  `MOVE_KING(12)` on promotion, then `TURN_END(2)` for the player who moved and
  `TURN_START(1)` for the next one; `mCurrentPlayer = (mCurrentPlayer + 1) % 2`.
- `OnReady` sends `INIT(0, 1)` (local id 0, red player 1) and then
  `TURN_START(0)`, so the black side moves first in the network convention.
- A side with no legal destination loses: `StartMove` returning false is how
  the movie reports "Won"/"Lost". No draw rule (move count, repetition) exists
  anywhere in the movie.

`CheckersNetworkEvent.as` ids: `INIT 0`, `TURN_START 1`, `TURN_END 2`,
`MOVE 10`, `MOVE_JUMPED 11`, `MOVE_KING 12`, `MOVE_INVALID 20`, `BOARD 30`,
`BOARD_STATE 31`, `VS_AI 32`, `PLAYERLEFT 40`, `WAITING 41`, `MUSTPLAY 50`,
`FORCEPLAY 51`, `LOCAL_* 100/101/102`.

Other recovered constants: `ConfigServer.STORAGE_NAME = "checkers_data"`,
`ConfigServer.GAME_ID = 21`, `ConfigClient.STRING_FILE = "CheckersStrings.txt"`,
`ConfigClient.END_GAME_DELAY = END_STAGE_DELAY = 3000`,
`GameCheckers.VERSION = "v1.12 May21"`, tutorial id `"11"`.

Transport text recovered from `GameClientNetwork.SendMove(start, end)`:
`AddMessage(true,"OnMoveMsg",start,end)` relays
`OnMoveMsg\t<start>\t<d1,d2,...>`; `GameServerNetwork` relays back
`OnCheckersEventMsg\t<type>\t<argA>\t<argB>`. The Free Realms movie bridge
(`SoeNetworkTypeFreeRealms`) forwards those tab-delimited strings through
`fscommand("FR_event","SendMsg(...)")`.

## Reconstruction boundary

Reconstructed (not original): the Rust session's replay protection and error
taxonomy, the `wire_index` numbering used for `TURN_*` arguments beyond the
recovered `INIT(0, 1)`/`TURN_START(0)` bootstrap, the instance-id namespace,
and all persistence and reward policy. The rules themselves (geometry,
compulsory capture, chain lock, promotion, no-move loss, event ids and event
order) are transcribed from the movie.

## Community corroboration (not authoritative)

- `lab/references/Sanctuary/src/Sanctuary.Packet.Common/LobbyGameDefinition.cs`
  documents the lobby game list with `4 - Chess` and `5 - Checkers`.
- `lab/references/FreeRealms-Legacy/src/Resources/Npcs.json` id 1404 is
  `Checkers & Chess` (`NameId 382885`, model `sg_gametable_01.adr`); the same
  resource lists the table in multiple regions. Those are world-table
  placements, not protocol evidence.
- Wiki corpus `lab/references/freerealms-wiki/wiki.sqlite`, article
  `Board Games` revision 16207 (2013-03-14) at
  `https://freerealms.fandom.com/wiki/Board_Games?oldid=16207`: "At launch,
  these tables only play Chess and Checkers".
- The pinned Ghidra client export contains no `checkers` symbol; the checkers
  rules ship only in the movie. The lobby/minigame opcodes
  (`BaseMiniGamePacket = 39`, sub-opcodes 14/16/18/47...) are documented in the
  community packet catalogue but the checkers-specific payload mapping is not
  recovered.

## Artifacts

- `prototype/osfr-tools/src/restoration/piece_009.rs` - self-contained
  `Board`, `Action`/`Jump`, `CheckersEvent`, `CheckersSession` (validation
  order: bounds, empty chain, finished, replay, turn, piece ownership,
  compulsory capture, chain continuation, destination), the movie event stream
  and the `OnMoveMsg` payload codec.
- `prototype/osfr-tools/tests/piece_009.rs` - 15 focused tests via
  `#[path = "../src/restoration/piece_009.rs"]`; no `lib.rs` edit.
- `scripts/pieces/piece_009.py` - recovery/checker: verifies the pinned table
  and movie hashes, extracts the six rows/links/locale, decompiles with pinned
  FFDec, verifies 38 rule markers and the numeric constants, and writes
  `checkers-config.json` / `findings.json` / `run.log`.
- `tests/pieces/test_piece_009.py` - 10 unittest checks, including three drift
  cases (missing row, tampered movie bytes, removed rule marker).
- Evidence: `lab/reports/piece-009-20261009T024655Z/` (`checkers-config.json`,
  `findings.json`, `run.log`, `scripts/`, `focused-red.log`,
  `focused-green.log`, `python-tests.log`).

Focused results:

```
cargo test --test piece_009        -> 15 passed; 0 failed; 0 ignored, 0 warnings
python -m unittest discover ...    -> 10 tests OK
scripts/pieces/piece_009.py        -> games=6 markers=38 scripts=55
```

RED evidence is retained: the first run failed with
`error: couldn't find file tests\..\src\restoration\piece_009.rs` (exit 101)
before the module existed.

## Integration contract (lead)

1. Register `pub mod piece_009;` (or include the file under
   `restoration`) and add an optional checkers field to the gameplay snapshot;
   persist the whole `CheckersSession` (it is `Clone` + `PartialEq`) before
   sending replies.
2. Bind the six TYPE 7 rows and the `Checkers & Chess` table NPCs to the
   session; map player ids to `Side` and keep the recovered
   `TURN_END`/`TURN_START` argument convention.
3. Feed client moves through `decode_move_payload` -> `apply`; send the returned
   events back through the movie bridge in order. Rewards stay in the lead-owned
   economy layer: this module never grants anything.
4. Translate a rejected `MoveError` into the recovered
   `MOVE_INVALID(20)` event when the gateway wants to inform the client; the
   typed error names which rule refused the move.

## Remaining gates and gaps

- Native/integration: lobby entry, table interaction, client movie launch,
  real move exchange, promotion/king visuals, end-of-game result panel, quit,
  reconnect and restart/replay behaviour.
- The binary packet carrying `OnCheckersEventMsg`/`OnMoveMsg` between the native
  client and the gateway is not recovered; only the movie-level text is.
- `MIN_PLAYERS`/`MAX_PLAYERS` are 0 in the shipped table, so matchmaking
  requirements are unrecovered.
- No draw rule and no original reward/loot definition were found; both stay
  explicit gaps rather than invented values.

## Skills applied

- `game-recon` / `game-research-websearch`: pinned community sources first
  (FFDec, Sanctuary, FreeRealms-Legacy) and the verified wiki corpus; the
  movie, not guesswork, is the rule source.
- `reverse-engineering`: decompiled the original movie and transcribed the
  server-relevant behaviour with script-level provenance.
- `ponytail`: one self-contained Rust file with std only, no new dependency, no
  speculative AI/powerup/reward engine; one focused test suite per language.
- RTK: normal commands first; evidence kept in task-owned lab files.
- Caveman: terse chat reporting only; this note stays complete.
