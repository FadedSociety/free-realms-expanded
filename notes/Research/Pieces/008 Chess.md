# 008 Chess

Status: launch/result contract recovered from shipped client artifacts; a bounded server-side
session model (legal moves, check/mate/stalemate, resign, reconnect) implemented and unit-tested.
No native client run was made by this worker, so nothing here is playable-accepted.

Queue item: `notes/WorkItems/008.md` (acceptance: recover launch/result contract; legal moves,
check/mate/stalemate and reconnect). Worker: crack_builder, DeepSeek V4.1 Flash, 2026-10-09 UTC.

## Owned artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_008.rs` | Recovered contract constants plus the bounded chess session model |
| `prototype/osfr-tools/tests/piece_008.rs` | 14 focused tests; `#[path]`-includes the module until the lead registers it in `src/lib.rs` |
| `scripts/pieces/piece_008.py` | Read-only evidence checker over the preserved lab corpus |
| `tests/pieces/test_piece_008.py` | 15 stdlib tests (fixtures + real-source checks, skipped without the lab) |
| `../lab/reports/piece-008-20261009T024347Z/` | Private evidence bundle: RED/GREEN logs, `chess-evidence.json`, the FFDec export of `game_chess.gfx` |

## Original facts

### Rows, type and assets

- `MiniGameData.txt` (sha256 `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252`) has
  **12 rows with `TYPE 6`**: ids 20, 65, 66, 67, 543-550. All share `NAME_ID 950`,
  `DESCRIPTION_ID 951`, `ICON_ID 917`, `MEMBERS_ONLY 0`, `MIN_PLAYERS 0`, `MAX_PLAYERS 0`;
  difficulties are 1/2/3/5.
- `MiniGameTypeData.txt` type 6 row: `IS_FLASH 0`, `IS_ACTIVE 1` - a native activity, not a Flash
  minigame, even though a SWF exists for presentation.
- Locale (`locale-id-to-text.json`, community reader output): 950 = "Chess", 951 = "Master the
  timeless tactics of Chess and defeat your opponent!".
- `GameTutorialData.txt` row 10 = `../minigames/chess/tutorial_chess.swf`, icon 7543, string 950;
  the movie's own `GameChess.mTutorial = "10"` matches.
- Pack catalogue: `AssetsW_010.pack` carries `game_chess.gfx` (entry 990), `tutorial_chess.gfx`
  (999), `game_chess.lst` (1031), `ChessAssets.txt` (807), `ChessMusic.ogg`, twelve `UI_Chess_*`
  sounds and the check/checkmate/checked cues. `game_chess.lst` records the build source
  `C:\dev\FreeRealms\Live\Runtime\Client\Minigames\Chess\game_chess.swf`.
- Asset hash used in tests: `game_chess.gfx` = 183,625 bytes, sha256
  `60787367f80af7450b5a79f7a1ab8ed52edd42c2b417d462fa0dc91e6f8f5099` (extracted copy
  `lab/extracted/client-assets/pack_10/000990.gfx`).
- `ChessLib.dll` (shipped, original archive member, sha256
  `0f28a061fc226ae11d22ec8c6265671b77dd55075a81ad92285898382b16c656`, 106,496 bytes, PE32
  machine `014C`) is a native chess engine: its strings include "rotated bitboards inconsistent",
  "Illegal move executed!", "PVS-%s" and "Error in internal iterative deepening, hash move was not
  there!". It is not managed .NET (ILSpy rejects it: "PE file does not contain any managed
  metadata").

### Client host ABI

Shipped `ScriptsBase.lua` (decompiled client UI):

```lua
function MiniGameFlash.commands.ChessInit(computer) MiniGameFlashC.ChessInit(computer) end
function MiniGameFlash.commands.ChessMovePlayer(from, to, promote, special)
  MiniGameFlashC.ChessMovePlayer(from, to, promote, special) end
function MiniGameFlash.commands.ChessMoveRequest(skill)
  local from, to, promote, special = MiniGameFlashC.ChessMoveRequest(skill)
  myMovie:Invoke("OnChessMoveResult", from, to, promote, special) end
```

`MiniGameStatusHandler.CHESS_TYPE = 6`. The native side implements the same three entry points:
`0x00c08010` (allocate `Chess`, `Chess::start(computer)`), `0x00c080f0` (read four flash arguments
into a `ChessMove`, `Chess::validate`, then `Chess::make_player_move`), `0x00c081a0`
(`Chess::make_computer_move`, return the four bytes to the movie).

### Result/turn contract (shipped movie, exported in the lab bundle)

`ChessNetworkEvent.as` declares the event codes and piece codes; `GameClientNetwork.as` /
`GameServerNetwork.as` name the messages; `ChessMachine.as` maps events to handlers;
`WaitingForUser.as` / `WaitingForNetwork.as` implement the two turn states.

| Code | Event | Payload as used |
| --- | --- | --- |
| 0 | INIT | two arguments; `argA == argB` means the local player is White ("Red") |
| 1 / 2 | TURN_START / TURN_END | turn ownership; `END_DELAY` 6000 user / 9000 network |
| 10 | MOVE | `fromIndex, toIndex` |
| 11 | CASTLE | `kingToIndex` in {2, 6, 58, 62} |
| 12 | PROMOTE | `fromIndex, toIndex, type` (piece code) |
| 13 | ENPASSANT | `fromIndex, toIndex, captureIndex` |
| 14 / 15 / 16 / 17 | CHECK / CHECKMATE / DRAW / STALEMATE | result signalling |
| 20 | MOVE_INVALID | `ViewItem.UndoLast()` |
| 30 / 31 | MOVE_REQUEST / MOVE_DATA | ask the local engine / deliver a move to it |
| 40 / 41 / 42 | PLAYERLEFT / WAITING / LOCAL_AI | departure, waiting text, local engine |

Piece codes (movie and native `piece_to_num` agree): 1 pawn, 2 rook, 3 knight, 4 bishop, 5 queen,
6 king. Board index is `y * 8 + x` with 0 = a1 (`Position.FromModelIndex`); the second player
mirrors with `63 - spot`. `ConfigServer.GAME_ID = 20`, `STORAGE_NAME = "chess_data"`;
`ConfigClient.END_GAME_DELAY = 3000`, `STRING_FILE = "ChessStrings.txt"`.

### Native rules semantics

- `Chess::start(int)` stores the computer side and clears the status word at `+0x1280`.
- `Chess::is_computer_move` = side-to-move == computer side; `Chess::check` = `in_check(side to
  move)`.
- `Chess::mate` = in check and no legal moves -> status = side + 2; `Chess::is_stalemate` = not in
  check and no legal moves -> status 1; `Chess::draw` -> status 1; `Chess::resign` -> status
  `flag + 4`; `Chess::is_draw` runs `check_draw` (repetition plus material test).
- `Chess::get_winner` maps status {2,4} -> 1, {3,5} -> 0, everything else -> 2.
  `Chess::is_resign` = status 4 or 5.
- `Chess::validate` rejects off-board squares, side mismatches, capture/promotion flag
  inconsistencies and castling to a destination other than the recorded target before
  `make_player_move` applies the move.
- Move byte: `[from, to, promote, special]`. `convert_player_move` sets `|0x80` for a two-square
  pawn push, `|0x60` when the destination is occupied, and the literal `100` for a pawn diagonal
  onto an empty square (en passant); `convert_player_promote` sets `|0x40`;
  `convert_castle_to_king` maps 1/2/8/0x10 to king targets 58/62/2/6.

### Result strings

`CodeStringMappings.txt`: `ChessCheck 19251`, `ChessMate 19252` ("Check Mate"),
`ChessMoveLast 19250`, `ChessMoveUser 19253`, `ChessMoveNet 19254`, `ChessPlayerLeft 3209`,
`ChessWaiting 3020` ("Waiting for the other player to start"),
`ChessErrorPlayerLeft 3336` ("The other player left. Close the chess game using the exit button in
the upper left corner."), `LeaderboardCategoryChess 3378`.

## Reconstructed (not historical)

- The server-side session model in `piece_008.rs`: session ids, position history, the
  `ChessSessions` store, reconnect snapshots and the resign-by-side API. The emulator server
  decompilation contains **no** chess handler, and no chess capture exists in `../lab`, so the
  original server's authority model is unknown. These are our policy shaped to the recovered
  client contract.
- `PLAYERLEFT` is modelled as "close the table, grant no winner", which follows
  `GameChess.OtherPlayerLeft` -> `EndOfGame("OtherLeft")` in the movie, but the original server's
  behaviour (and whether it re-paired players) is not recovered.
- Insufficient material and threefold repetition reproduce `Chess::check_draw`'s shape (pawns on
  either side gate repetition; the material test uses minors only), not a literal port of the
  bitboard counters.

## Gaps (explicit)

- The SOE wire frames that carried these messages (launch, move relay, result) are unrecovered;
  the documented packet families are enums only (`BaseMiniGamePacket 39` with sub-opcodes
  `MiniGamePayloadPacket 14`, `MiniGameInfoPacket 16`, `MiniGameGameOverPacket 18`,
  `MiniGameGameEndScorePacket 47`, `MiniGameGroupInfoPacket 50`; `PacketInviteAndStartMiniGame
  180`; `ClientActivityLaunchBasePacket 175`).
- Which movie dialogue invokes `ChessInterface.resign`, and with what argument: `resign` and
  `is_resign` exist as native strings, and `Chess::resign` writes status `flag + 4`, but the caller
  is inside `game_chess.swf` frames not exercised here.
- Reward, rating and leaderboard amounts for `TYPE 6` rows; `LeaderboardCategoryChess 3378` only
  names the category.
- Original AI strength/behaviour: `ChessLib.dll` and the native search are present, but no
  difficulty-to-depth mapping was recovered for the four difficulty codes.
- Native acceptance: no client run; the module is not registered in `src/lib.rs`, and no gameplay
  or gateway wiring exists for chess yet.

## Verification

```powershell
# RED before implementation: 14 tests, 0 passed, exit 101
# (lab/reports/piece-008-20261009T024347Z/cargo-test-piece_008-RED.log)
Set-Location prototype\osfr-tools
..\..\scripts\cargo.ps1 test --test piece_008

# GREEN after implementation and rustfmt: 14 passed, 0 failed, exit 0
..\..\scripts\cargo.ps1 test --test piece_008

# whole workspace suite: all targets ok, exit 0 (lab/reports/.../cargo-test-all.log)
..\..\scripts\cargo.ps1 test

# evidence checker against the preserved corpus: rows=12 type=6 host_abi_missing=0 symbols=805,
# checks passed, exit 0
& ..\lab\tools\wiki-env\Scripts\python.exe scripts\pieces\piece_008.py --check `
  --out ..\lab\reports\piece-008-20261009T024347Z\chess-evidence.json `
  --movie-scripts ..\lab\reports\piece-008-20261009T024347Z\chess-movie\scripts\scripts

# python tests: 15 tests, OK (lab/reports/.../python-tests-piece_008.log)
& ..\lab\tools\wiki-env\Scripts\python.exe -m unittest discover -s tests/pieces -p "test_piece_008.py" -v
```

The Rust suite covers: the recovered row/type/tutorial constants, event and piece code tables, the
native status/winner switch, the 20 legal opening moves, illegal-move rejection, moving into check,
castling rights and attacked paths, en passant, promotion, the recovered move-byte encoder, fool's
mate, stalemate, insufficient material, threefold repetition, turn alternation, post-result lock,
resign, reconnect snapshots and unknown-session rejection.

## Skills applied

`game-recon` (engine classification: native type-6 row with a presentation SWF), `game-research-websearch`
practice (community readers first: pinned Ghidra export, FFDec on the shipped pack asset, the
read-only wiki corpus), `reverse-engineering` (native pseudocode symbols, `ChessLib.dll`
fingerprint, movie ActionScript export), RTK (normal commands first), Ponytail (reuse the pinned
lab tools and the shipped tables instead of inventing new readers) and Caveman (concise
communication). `asset-pipeline` and `game-automation` were not needed: no asset conversion, and
the shared native window is lead-controlled.

Cross-links: [[../MMO Work Queue]], [[../Original Activity Coverage]].
