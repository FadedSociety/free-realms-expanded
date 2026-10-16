---
type: build-guide
section: Gameplay
updated: 2026-10-12
---

# Chess

Part of [[Build Guides]] › Gameplay. Related: [[Minigame Framework]].

<!-- NARRATIVE START -->
## How it works

**How to read this.** Confidence words (`confirmed`, `likely`, `speculative`) are the native-map labels for a function. Source tags: *client* = read from the shipped executable, in a shard or in this pass (pseudocode and bytes; the engine functions in the failed shard S0526 are tagged *client (S0526)* because they were read raw and never reviewed); *client movie* = the 103 ActionScript files exported from the shipped `game_chess.gfx` (pack entry 990, 183,625 bytes); *client Lua* = the decoded `ScriptsBase.lua`; *client resource* = the loose tables in `Resources/` (`MiniGameData.txt`, `MiniGameTypeData.txt`, `GameTutorialData.txt`, `CodeStringMappings.txt`) and the pack manifest; *client locale* = the shipped text table; *video* = a reviewed frame record (names and chat never recorded); *wiki* = the fan wiki; *vault* = the research note `Research/Pieces/008 Chess.md`, used only for facts it took from the client (its server session model is ours and is not used here). Precedence is client, then video, then wiki. *(R)* marks reconstructed policy that is ours, not original behaviour. No original server code and no chess packet capture exist, so everything said about the server is a hypothesis built from what the client requires. **Not used:** the community packet catalogue, any emulator or legacy-server code. Related: [[Minigame Framework]], [[Matchmaking and Lobby]], [[Rewards and Collections]], [[Quests and Objectives]].

**Colour and board facts used throughout** (lead-verified from `init_game` and the generators, S0527 and S0528 correction notes): **white is positive, black is negative; side 0 is white.** Bit index equals square index, row 0 is rank 8 (`rank = 8 - (sq >> 3)`), so a8 = 0, h8 = 7, e8 = 4, a1 = 56, e1 = 60 (`0x3c`), h1 = 63. Negative pieces start on rows 0 and 1, positive pieces on rows 6 and 7. The first S0528 findings said the opposite and the stored `does` text of many S0528 records (the `init_*` setters, `in_check`, `pinned_to_square`, `gen_attacks`, `gen_pawn_key`, `gen_zob_key`, the three evaluators) still carries inverted colour or rank labels; the correction notes in the review records and at the end of each `findings.md` are right and are what this guide follows.

### Overview

Chess is a board-game minigame played at the "Checkers & Chess" tables found in many towns. It is not a world activity like racing: the player sits at a table and a Flash movie (`game_chess`) takes over a minigame window. The same host also runs Checkers (type 7). *client Lua* (`CHESS_TYPE = 6`, `CHECKERS_TYPE = 7`), *client resource*.

- **Tables.** On the world map and as purple activity markers the table is called "Checkers & Chess" (client locale 1966607816 and 2056434849; the quest text points to "the nearest game table"). Video shows the marker in Merry Vale, Shrouded Glade, Sweetwater Climb, Seaside, Blackspore and the Sasparilla Brewery area (128 frames in 47 videos, all world frames with a distance in metres). The wiki lists the table on the Sanctuary, Sacred Glade, Blackspore Swamp, Lakeshore and Robgoblin Camp pages and says "at launch, these tables only play Chess and Checkers". *video, wiki*
- **Modes and difficulty.** There are 12 minigame rows of type 6, all named "Chess" (string 950, "Master the timeless tactics of Chess and defeat your opponent!"). Nine are one-player games against the computer in three difficulties (Easy, Medium, Hard, `DIFFICULTY` 1, 3 and 5) and three are two-player games (rows 65, 546, 550). The row names in the locale say "random order" for the one-player rows, which reads as random choice of colour (R), and "board 1", "board 2" or "board 3" (see Data). *client resource + client locale*
- **What the player sees.** An 8 by 8 board, pieces picked up and dragged, rollover highlights that show every legal square for the hovered piece, a captured-pieces list, a move indicator with the strings "Your move", "Wait for the other player", "Last move", "No moves yet", "Check!" and "Check Mate", and an exit button in the upper left. The board is labelled A to H and 8 to 1 for white and mirrored for black. Check, checkmate, capture, pick-up and put-down each have their own sound, plus a music loop. *client movie + client locale + client resource*
- **Tutorial.** When a game starts the movie asks the host to show the tutorial movie (`tutorial_chess.swf`, tutorial row 10; whether the host shows it again is its own "show again" choice) with pages for the objective, movement, each piece, castling, promotion, check and en passant. The movie also raises the matching page the first time a castle, promotion, check or en passant happens. *client resource + client movie + client Lua*
- **Leaving.** There is no resign button, draw offer, clock or take-back in the movie. A player leaves with the exit button. If the other player leaves, the survivor sees "The other player left. Close the chess game using the exit button in the upper left corner." *client movie + client locale*
- **Rewards and goals.** No reward amount for chess is recoverable. All 12 rows have `LEADERBOARD_ID 0` although a "Chess" leaderboard string exists. Goal texts: the achievement "Chess Master" (client locale; wiki says "Win a game of Chess", bronze trophy, 5 points), "Win a game of chess", "Beat singleplayer Chess ten times". Quests send the player to chess: an NPC challenge in Briarwood ("Beat Feari at Chess and join the Big Brain club"), and chains that deliver a handcrafted chess table to a camp or to Solace after a win or a high score (content; see [[Quests and Objectives]]). *client locale, wiki*
- **What our server already does.** Nothing for chess. The vault note 008 holds a rules-and-session model written by an earlier worker, not registered in the build and not used here. `Original Activity Coverage` lists all 12 chess rows as having no captured receipt.

### How the client implements it

Three layers cooperate. Knowing which one owns a decision is the whole design.

| Layer | What it owns | Evidence |
|---|---|---|
| The chess movie (`game_chess`, ActionScript 2) | All drawing and input; a complete rules model for rollovers (move generation with check filtering for the user's own pieces); the turn state machine; the text protocol to the server. | client movie |
| The Lua host (`MiniGameFlash`, `MiniGameFlashC`) | Passes the movie's `FR_event` commands to the engine bindings and to the server; hands server messages to the movie. | client Lua |
| The native engine (`Chess`, statically linked in `FreeRealms.exe`) | A bitboard chess program used only for the computer opponent: it can accept a human move into its position and produce a computer move. Nothing else is asked of it. | client |

**Who decides moves.** The server arbitrates everything; the client engine only proposes moves for the computer side. The movie asks the engine for a move only when the server sends event 30 (`MOVE_REQUEST`) and returns the answer to the server (`OnMoveComputerMsg`), and the engine learns the human's move only when the server sends event 31 (`MOVE_DATA`). The native handlers return nothing about legality (`ChessMovePlayer` returns 0 whether the move was accepted or silently dropped), and no code in the executable reads the engine's result code at `+0x1280`; `mate`, `is_stalemate`, `is_draw`, `get_winner`, `is_over`, `resign` and the `convert_*` helpers have no callers in the executable. So the client cannot tell the server who won, and the server must run its own chess rules for check, checkmate, stalemate and draws. *client, client (S0526), client movie*

The engine shipped in `FreeRealms.exe` is also present as a separate `ChessLib.dll` (106,496 bytes, exports the same `Chess::` symbol set) in the client folder. The executable contains no reference to the string "ChessLib" and all six engine error strings are present in both files, so the executable runs its own copy. Whether the two builds are byte-for-byte equivalent was not compared. *client*

#### Main engine classes

| Class | Role | Evidence |
|---|---|---|
| `Chess` (0x1290-byte object; ctor `00f4eb70`, `start` `00f49170`) | The whole engine instance: position, bitboards, history, search state, book handle. One global pointer `DAT_01bec3b0`. | confirmed |
| `ChessPosition` (base of `Chess`, 0x76 dwords = 472 bytes) | The part of the object that is copied to snapshot a position (`ChessPosition::Assign` `00c07ed0`). | likely |
| `ChessMove` (4 bytes: from, to, promote, special) | The move value passed from Lua, stored in lists and in the transposition table. | confirmed |
| `ChessMoveData` (8 dwords), `ChessControls` (15 dwords) | Per-move context and control masks; only their copy assignments are mapped. | confirmed / likely |
| `ChessLine` (count plus up to 100 moves, 0x65 dwords) | A principal variation. | likely |
| `MiniGameFlashC` Lua module (table at `0x0182a1e8`) | Binds `ChessInit` (`00c080b0`), `ChessMovePlayer` (`00c080f0`), `ChessMoveRequest` (`00c081a0`), `LoadData`, `SendMessage` (`00c08480`) and `RequestTintByName`. | confirmed (S0383) |

#### Game flow (reconstructed from the movie, the Lua host and the engine)

1. **Launch.** The player uses a table; the minigame framework starts one of the type 6 rows and the client shows the `MiniGameFlash` window with the chess movie (`MiniGameFlash:Show(swf, ...)`). How the movie name reaches the window is framework work (see [[Minigame Framework]]). The Lua treats `CHESS_TYPE` like `FLASH_TYPE` for HUD and button states. *client Lua*
2. **Connect.** When sounds and preloader finish, the movie "connects": it sends the message `OnConnectMsg` to the server through `SendMsg`. *client movie*
3. **Ready, tutorial, start, init.** The server sends `OnServerReadyMsg` (music starts, the movie resets and asks the host to show the tutorial). When the host reports the tutorial closed (or skips it), the movie resets again and sends `OnStartGameMsg` with argument 0. The movie code then waits for chess event 0 (`INIT`); that the server sends `INIT` in answer to `OnStartGameMsg` is the natural reading (R). If the two arguments of `INIT` are equal the local player is white ("Red") and moves first; otherwise the local player is black, the board labels flip and the movie waits. *client movie*
4. **One-player setup.** The server sends event 42 (`LOCAL_AI`, one argument). The movie calls `ChessInit(argument)` through the host. Native `Chess_CreateAndStart` frees any old engine, allocates 0x1290 bytes, runs the constructor with a **null** book path, stores the argument at `this+0x13` and clears the result code. *client, client (S0526)*
5. **A turn.** The server sends `TURN_START` (1). The user's side shows "Your move" and turns on rollovers; the other side shows the opponent's indicator. The user drags a piece; the movie applies it locally and sends one of `OnMoveMsg`, `OnCastleMsg`, `OnPromoteMsg` or `OnEnPassantMsg`. The server answers with `MOVE_INVALID` (20) to make the movie undo the move, or relays the move to the other side as event 10, 11, 12 or 13 and ends the turn with `TURN_END` (2). *client movie; server side (R)*
6. **Computer reply (one player).** After a human move the server (R) sends event 31 (`MOVE_DATA`) with the human's move in engine bytes, which runs `validate` then `make_player_move` in the engine, then event 30 (`MOVE_REQUEST`) with a `skill` argument. The host calls `make_computer_move(skill)`, gets four bytes back and invokes the movie function `OnChessMoveResult`, which sends them to the server as `OnMoveComputerMsg`. The server turns them into a normal move event for the movie. *client movie, client Lua, client*
7. **Check, mate and end.** The server sends `CHECK` (14) and `CHECKMATE` (15) and the movie plays the matching stinger and indicator. A win or loss is `OnGameWonMsg` or `OnGameLostMsg`; on either the movie sends `FRServer_GameOver` at once (it also queues a second send after 3 seconds, but the same handler clears its event queue, so the second send never fires), and sends `FRServer_GameClose` one second after the player clicks through the end screen. `OnGameStoppedMsg` only resets the movie and sends nothing. The exit button calls the host `Quit`, which leaves the minigame through the framework. *client movie, client Lua*
8. **Opponent leaves.** Event 40 (`PLAYERLEFT`) shows the error text, and when the player dismisses it the movie ends the game with "OtherLeft" and sends `FRServer_GameOver` one second later. Event 41 (`WAITING`) shows "Waiting for the other player to start". *client movie*

#### The engine

**Board representation.** The object holds the board as 64 signed dwords at `+0x14` (index = square, piece codes 1 pawn, 2 rook, 3 knight, 4 bishop, 5 queen, 6 king, negative for black) and, alongside it, 64-bit piece bitboards at `0x170 + piece*8` (so `+0x148..0x168` are black queen, bishops, knights, rooks and pawns, and `+0x178..0x198` are white pawns, rooks, knights, bishops and queen), occupancy words `+0x1a8/+0x1ac` (black) and `+0x1b0/+0x1b4` (white), and rotated occupancy words used for sliding attacks: `+0x1b8` (90-degree rotation, for files), `+0x1c0` (one diagonal direction) and `+0x1c8` (the other). King squares are ints at `+0x134` (white) and `+0x138` (black); a captured king is `-1`. Sliding attacks come from lookup tables indexed by square times 256 plus an 8-bit occupancy slice (rook rank `DAT_01c5ed28`, rook file `DAT_01c368f0`, bishop `DAT_01c16240` for the +/-9 diagonal and `DAT_01c7eda8` for the +/-7 diagonal), with king, knight, between-squares and ray tables built at construction time by `init_const_boards`. *confirmed / likely (S0527, S0528)*

**Move encoding.** A move is four bytes `[from, to, promote, special]`. `promote` uses the piece codes 2 rook, 3 knight, 4 bishop, 5 queen. The `special` byte takes these values: 0 for a quiet piece move, `0x60` for any capture (pawn or piece), `0x64` for an en passant capture, `0x40` for a pawn push or a promotion push, `0xc0` for a two-square pawn push, and the castling flags `0x01` white queen side, `0x02` white king side, `0x08` black queen side, `0x10` black king side. (`validate` reads the byte as flags: `0x20` capture, `0x40` pawn move, `0x04` en passant, `0x80` double push, with `0x60` as capture plus pawn-move bits.) A castle is always stored as `[0, 0x3e, 0, flag]` for both sides; the flag alone says which. *confirmed (S0527); `convert_player_move` agrees (S0526)*

**Move generation.** Pseudo-legal generators per piece and side (`w_*` and `b_*`, captures and quiet moves separately, plus castling and check evasions) fill a move list. `valid_moves` builds the candidate list for the side to move, then walks it from last to first, plays each move, rejects it if the mover's king is attacked, and undoes it, so the legal list comes out in **reverse generation order**. Pawn capture generators already emit capture-promotions (four moves with promote 2 to 5, flag 0x60), and the separate `*_pawn_promotions` generators emit the promotion pushes and emit the capture-promotions again; `valid_moves` calls all three, so **promotion moves appear twice** in its output (`perft` avoids this with the `_non_promotions` capture lists and counts each promotion once). The earlier S0527 note that perft skips promotion captures was wrong. *confirmed / likely (S0527, correction note)*

**Validation of a human move.** `validate` is the gate used by the Lua handler. It is **pseudo-legal only**: it rejects out-of-range squares, uses a 13-plane lookup table (`DAT_01c06fe0`, index `piece*0x1000 + from*64 + to`, built by `init_val_table`) to classify the move (rank, file, diagonal, knight or king step, pawn push, double push, capture or castle) and then checks ownership (skipped when a castling flag is set), blockers, the capture and en passant flag rules and, for castling, that `w_castles` or `b_castles` generates a move with the same flag byte. It never tests whether the mover's own king is left in check, and a rejected move is dropped silently. *likely, with the S0528 review corrections*

**Search.** `make_computer_move(skill)` runs in this order: `check_draw` (a draw sets the result code to 1 and returns the zero move); a roll `random_32 % 100`, and if the roll is above `skill` it tries `get_random_capture`; the opening-book probe; then `search`. `search` is iterative deepening with a full window of plus or minus 1,000,000, depth counted in 60ths of a ply (iteration n searches `n*60 + 30`), up to 89 iterations, stopping at the clock or the stop flag or on a mate-range score (`>= 0x7f9b`). `negamax` is a fail-soft principal-variation search with a 1,024-entry, 16-byte-entry transposition table, a check extension, null-move pruning (when more than two plies remain and the side to move is more than 299 up in material; reduction two plies, or three plies when at least three plies remain and the lead is 500 or more), internal iterative deepening, captures ordered by static exchange (`see_sort`), two killer moves per ply, history ordering (64 by 64 counters, quartered at the start of every search), and `quiescent` at the leaves (captures ordered by SEE, check evasions when in check, at most 0x41 plies). Two ordering sorts are partial: `history_sort_moves` does only three bubble passes, and `mvv_lva_sort` and `see_sort` stop at the first negative score, which prunes losing captures. Mate scores are `0x7fff - ply`. *confirmed (S0529 round 2 corrections)*

**Evaluation.** `score` first computes the material difference (white minus black, negated for black to move). When the material difference lies more than 101 outside the alpha-beta window it returns the material difference alone. Otherwise it chooses an evaluator by the number of non-pawn pieces on the board (sum of the counters at `+0x11c..+0x128`): 12 or more `opn_score`, 5 to 11 `mid_score`, under 5 `end_score`. All three score white-positive from the board array: pawn 100 plus a piece-square table (separate white and black tables) with passed, backward, doubled and isolated adjustments; knight 310 and bishop 325 plus tables; rook 500, raised to 508 (`0x1fc`; 512 in `end_score`) when it stands on the **opponent's second rank** (rank 7 for white, rank 2 for black), plus 5 or 8 for open or half-open files; queen 900 (896 in the opening while the queen has left d1 or d8 and minor pieces are still at home); king tables (separate middlegame, per-colour, and endgame tables) with castling-state and pawn-cover terms; blocked-centre-pawn terms (7 in the opening, 5 in the middlegame); and, in the opening and middlegame evaluators, a king-safety block when the kings are more than two rows apart. Only `mid_score` and `end_score` add the plus or minus 10 term, and it fires only when `DAT_01ca2a0c` (the current non-pawn piece count, written by `score`) exceeds `DAT_01ca2a08` (the same count at the search root, written by `search`), which can only happen after a promotion inside the search (R reading of the two writers). `opn_score` never reads either. *likely (S0528 review)*

**Difficulty.** The only difficulty input the client gives the engine is the `skill` argument of `ChessMoveRequest`, used as a percentage threshold: with probability `(99 - skill) / 100` the computer first tries a random capture (a random entry from the capture-only list, counted back from the end; a roll of 0 in 64 picks the empty slot, so it then falls through to the search). A value of 100 or more never takes the random-capture path; a low value takes a random capture nearly every time one exists, and plays the search otherwise. The search time parameters `this+0x204` and `this+0x210` are zeroed by the constructor and **no code in the chess range ever writes them**, so the budget given to `search` is zero: the loop stops at the first iteration boundary at which the millisecond clock has moved, which means only a few plies (R inference, not observed). The server's choice of `skill` per row is therefore the entire original difficulty control that we can see, and the original values are unknown. *client, client (S0526)*

**Draws, results, book and randomness.** `check_draw` calls a draw for a repeated position (two earlier matches, or one earlier match inside the search), for a ply counter of 100 or more, and for insufficient material (no pawns and little else). Because `make_player_move` zeroes the ply counter and restarts the key history on every human move, repetition across real moves can hardly be detected by the client engine; only the in-search form and the material test matter in play (R reading). The engine's result code at `+0x1280` is 0 running, 1 draw or stalemate, 2 white mated, 3 black mated, 4 and 5 resignation of side 0 or 1; it is set by `mate`, `is_stalemate`, `draw`, `resign` and `make_player_move`/`make_computer_move`, but nothing reads it. The opening book (`init_book`, `use_book`) is **dead code in this client**: the constructor is always called with a null path, so `init_book` never opens a file and `use_book` returns 0 immediately. The random generator `random_32` is an additive lagged-Fibonacci generator seeded from a constant table on first use (`DAT_01b95cd0` starts at 1, no clock seed), shared across games for the life of the process, so the computer's random choices repeat the same sequence on every run. *confirmed / likely, client (S0526)*

### Data and formats

#### Minigame rows (client resource and locale)

All 12 rows share `TYPE 6`, `NAME_ID 950`, `DESCRIPTION_ID 951`, `ICON_ID 917`, `MEMBERS_ONLY 0`, `MIN_PLAYERS 0`, `MAX_PLAYERS 0`, empty `SHORTCUT` and `PRELOAD_FILE`, `START_SOUND_DEFINITION 0` and `LEADERBOARD_ID 0`. The type row (`MiniGameTypeData.txt`, type 6) has `IS_FLASH 0`, `IS_MICRO 0` and `IS_ACTIVE 1`. The locale carries a working name for each row:

| Row | Working name in the locale | `DIFFICULTY` |
|---|---|---|
| 20 | random order, board 1, easy | 1 |
| 65 | 2 player, board 1 | 1 |
| 66 | random order, board 1, medium | 3 |
| 67 | random order, board 1, hard | 5 |
| 543 | random order, board 2, easy | 1 |
| 544 | random order, board 2, medium | 3 |
| 545 | random order, board 2, hard | 5 |
| 546 | 2 player, board 2 | 2 |
| 547 | random order, board 3, easy | 1 |
| 548 | random order, board 3, medium | 3 |
| 549 | random order, board 3, hard | 5 |
| 550 | 2 player, board 2 (the text says board 2; rows 547 to 549 are board 3) | 2 |

The same locale also holds unused placeholders "Chess: 1p white", "1p black", "1p random" and "2p". Other client resource facts: tutorial row 10 is `../minigames/chess/tutorial_chess.swf` (icon 7543, string 950); the pack manifest lists `game_chess.gfx` (183,625 bytes) and `game_chess.swf` (631,020 bytes), the tutorial files, `ChessAssets.txt`, music `ChessMusic.ogg`, five `Chess_MX_*` stingers (check, checked, checkmate, checkmated, checkmated short), nine `UI_Chess_*` effects (capture, opponent slide loop, mouse over, three pick-ups, three put-downs), two images (`chess_detail`, `chess_thumb`) and the chess category and lobby icons in three sizes each. The movie's own constants are `GAME_ID = 20` (equal to row 20), `STORAGE_NAME = "chess_data"`, `STRING_FILE = "ChessStrings.txt"`, `END_GAME_DELAY = 3000` ms. `LEADERBOARD_ID` is 0 for every row. *client resource, client movie*

#### Strings

`CodeStringMappings.txt` binds the movie's string names: `ChessMoveLast` 19250 "Last move", `ChessCheck` 19251 "Check!", `ChessMate` 19252 "Check Mate", `ChessMoveUser` 19253 "Your move", `ChessMoveNet` 19254 "Wait for the other player", `ChessMoveNone` 130773 "No moves yet", `ChessWaiting` 3020, `ChessPlayerLeft` 3209, `ChessErrorPlayerLeft` 3336, the leaderboard category 3378, and twelve `ChessTutorial*` pages (92288 to 92311). *client resource + client locale*

#### Coordinates and conversion

The movie, the wire and the engine share one board frame: **index 0 is a8 and index 63 is h1, white's king starts on 60 and black's on 4**. The vault note says index 0 is a1; the movie's own setup code (white king at row 7, column 4, black king at row 0, column 4, castling targets 2, 6, 58, 62) and the engine agree on a8 = 0, so the note is wrong on this. A black player's movie draws its own pieces at the bottom, so the movie converts every index it sends or receives with `63 - index`; the wire always carries the white-at-bottom frame. *client movie, client*

| Wire event | Arguments | Engine bytes `[from,to,promote,special]` |
|---|---|---|
| move (event 10 / `OnMoveMsg`) | from, to | `[from, to, 0, flags]` with flags per the encoding above |
| castle (11 / `OnCastleMsg`) | king destination 2, 6, 58 or 62 | `[0, 0x3e, 0, flag]`; 2 gives 0x08, 6 gives 0x10, 58 gives 0x01, 62 gives 0x02 |
| promote (12 / `OnPromoteMsg`) | from, to, piece 2 to 5 | `[from, to, piece, 0x40 or 0x60]` |
| en passant (13 / `OnEnPassantMsg`) | from, to, captured pawn square | `[from, to, 0, 0x64]` |

The castling and promotion conversions come from `convert_player_castle`, `convert_castle_to_king` and `convert_player_promote` (client (S0526)); the executable never calls them, so a server that feeds the engine must apply them itself. The movie always promotes the user's pawn to a queen (`CreateMoveOrPromotion`) but accepts any of the four pieces from the network. *client (S0526), client movie*

#### Engine object layout (`Chess`, 0x1290 bytes)

| Offset | Contents |
|---|---|
| +0x00 / +0x08 | position key and pawn key (64-bit each) |
| +0x10 | side to move (0 = white) |
| +0x11 | en passant square (the square the capturing pawn lands on), 0 when none |
| +0x12 | castling rights (0x1b at the start; bits 1 and 2 white, 8 and 0x10 black) |
| +0x13 | argument of `start` (see Open questions for its meaning) |
| +0x14 | board, 64 signed dwords |
| +0x114 / +0x118 | black and white material totals |
| +0x11c..+0x128 | non-pawn piece counters (phase) |
| +0x12c / +0x130 | white and black pawn counts |
| +0x134 / +0x138 | white and black king squares |
| +0x148..+0x19c | piece bitboards (`0x170 + piece*8`) |
| +0x1a8 / +0x1b0 | black and white occupancy (lo word, hi word follows) |
| +0x1b8, +0x1c0, +0x1c8 | rotated occupancy words |
| +0x1e8 | searches since the last book hit; +0x1ec move counter |
| +0x204, +0x210 | time parameters, never written |
| +0x214 | ply counter (byte); +0x218 key history; +0x5d8 history pointer |
| +0x5e0 | 100 records of 0x20 bytes (role not established) |
| +0x1260..+0x127c | record pointer, stop flag, iteration depth, node counters, mate distance |
| +0x1280 / +0x1284 | result code, last `make_move` result |
| +0x1288 | opening-book `FILE*` |

*confirmed / likely (S0527, S0528, S0529)*

#### Hash and book formats

- **Zobrist keys.** One flat array of 64-bit keys: positive piece `p` (white) at `01ca0be0 + p*0x400 + sq*8`, negative piece (black) at `+0x200`; pawn-only keys at `01ca0fe0` and `01ca11e0`; four castling keys at `01ca29e8`, `01ca29f0`, `01ca29f8`, `01ca2a00`; en passant keys at `01ca27e0`; side-to-move key `change_side_zob`. `init_zobrist` fills the array rank-reversed with 12 tables of 0x200 bytes. *confirmed (S0528 correction)*
- **Transposition entry (16 bytes).** move dword at +0, 16-bit score at +4, depth byte at +6, flag byte at +7 (2 lower bound or beta cutoff, 4 upper bound, 0 exact), position key at +8 and +0xc. Table size is `1 << floor(log2(DAT_01b964f8))`, and `DAT_01b964f8` is `0x400`, so 1,024 entries. A second 16-byte-entry table (`DAT_01caf548`, the pawn table) is allocated, but its writer `add_move_to_pt` has no caller and nothing reads it. *confirmed (S0529 round 2)*
- **Opening book file (unused).** A table of 4-byte offsets indexed by `(key_hi >> 17) * 4`; each bucket is a count then 20-byte records (bytes 0 to 7 key, low 24 bits of bytes 8 to 11 weight, bytes 12 to 19 ignored); only weights above 5 count and one is chosen with `random_32` modulo the total weight. No book file name appears anywhere. *confirmed (S0529)*
- **Piece values for exchange ordering** (`DAT_01b964dc`): pawn 100, rook 500, knight 300, bishop 300, queen 900, king 40000. *client bytes*

### Network

**Transport.** Chess has no packet class of its own. Every message between the movie and the server is a **text string carried by the generic Flash minigame payload packet**: family 39 (`BaseMiniGamePacket`), sub-opcode 14 (`MiniGamePayloadPacket`). *client*

- **Movie to server.** The movie calls `fscommand("FR_event", "SendMsg(\"<name>\t<arg>\t...\")")`. The Lua `SendMsg` command forwards the string to the native `SendMessage` (`00c08480`), which calls `SendMiniGameFlashPayload` (`009be6b0`). That builds `BaseMiniGamePacket(0x0e, stateId, -1, -1)` with the current minigame state id (zero if none), appends the string bytes **plus a terminating zero**, and sends it only when the connection flag is set. *client, client Lua, client movie*
- **Server to movie.** The client reads the five-field mini-game header (a 16-bit value, an 8-bit value and three 32-bit values), then a 32-bit length and that many bytes, and raises the Lua event `MiniGameFlash:OnMessage` with the string (it is held in what looks like a fixed 128-character Lua parameter, so keep messages short; likely). The Lua calls the movie's `OnMessage`, which splits the string on tab characters: the first field is a function name looked up on the movie's network object, the rest are arguments converted by `ConvertArg` (`undefined`, `true`, `false`, a number or a string). *client (`009b89e0`), client Lua, client movie*
- The wire byte layout of the outer family-39 packet beyond the header fields above, the meaning of the five header fields, and the state id's source belong to the framework and were not read for chess. *client*

**Messages the movie sends (text name, arguments).**

| Message | Arguments | When |
|---|---|---|
| `OnConnectMsg` | none | After preload and sound are ready |
| `OnStartGameMsg` | 0 | After the tutorial window closes (or is skipped) |
| `OnMoveMsg` | from, to | The user made a normal move |
| `OnCastleMsg` | king destination | The user castled |
| `OnPromoteMsg` | from, to, piece | The user promoted (always queen from this movie) |
| `OnEnPassantMsg` | from, to, captured square | The user captured en passant |
| `OnMoveComputerMsg` | four engine bytes | Reply to `MOVE_REQUEST` |
| `FRServer_GameOver` | none | At once on a win or loss message; one second after the player dismisses the "other player left" error |
| `FRServer_GameClose` | none | One second after the player clicks through the end screen |
| `FRServer_Pause` | true or false | If the framework pauses the movie |

Indices in these messages are in the white-at-bottom frame. The movie also sends the non-message host command `Quit()` from the exit button; the Lua turns it into the framework leave. *client movie, client Lua*

**Messages the server sends.**

| Message | Arguments | Effect |
|---|---|---|
| `OnServerReadyMsg` | none | Music on, reset, tutorial on first game |
| `OnChessEventMsg` | type, up to four arguments, then a label | The chess events below |
| `OnGameWonMsg`, `OnGameLostMsg` | none | Ends the game for the local player; the movie sends `FRServer_GameOver` back |
| `OnGameStoppedMsg` | none | Resets and ends the movie; nothing is sent back |
| `OnPauseMsg`, `OnActiveMsg` | state | Framework pause or focus |

**Chess events** (`OnChessEventMsg`, first argument is the event type; arguments are the next four tab fields):

| Type | Name | Arguments | Movie behaviour |
|---|---|---|---|
| 0 | `INIT` | A, B | A = B means local player is white and moves first; otherwise black and waits |
| 1 | `TURN_START` | none | User state: "Your move" and rollovers. Network state: opponent indicator |
| 2 | `TURN_END` | none | Switches between the user state and the network state (after any animation) |
| 10 | `MOVE` | from, to | Opponent move animates |
| 11 | `CASTLE` | king destination | Opponent castles (king and rook animate) |
| 12 | `PROMOTE` | from, to, piece | Opponent promotes |
| 13 | `ENPASSANT` | from, to, captured | Opponent captures en passant |
| 14 | `CHECK` | none | Stinger and indicator |
| 15 | `CHECKMATE` | none | Mate stinger and indicator; also stores a 6 s (user state) or 9 s (network state) end timer value that nothing in the movie reads |
| 20 | `MOVE_INVALID` | none | User state only: undo the dragged move |
| 30 | `MOVE_REQUEST` | player, skill | Ask the local engine for a move (`skill` is the only argument used) |
| 31 | `MOVE_DATA` | four engine bytes | Feed a move into the local engine |
| 40 | `PLAYERLEFT` | none | Error text "Other player left" |
| 41 | `WAITING` | none | "Waiting for the other player to start" |
| 42 | `LOCAL_AI` | computer | Start the local engine |

The movie names the arguments of types 30 and 31 and of its reply (`to, from, promote, special`) the wrong way round, but the four values pass through in engine order unchanged (the first is the `from` byte). Types 16 (`DRAW`) and 17 (`STALEMATE`) are declared in the movie's event table but have **no handler in the event map**, so the movie logs an unknown event and ignores them. Draws and stalemates therefore cannot be shown by an event; the server has to end such a game itself (see Open questions). *client movie*

**Reconstructed sequences (R).**

- *One player.* `OnConnectMsg` > `OnServerReadyMsg` > (tutorial) > `OnStartGameMsg 0` > `INIT` > `LOCAL_AI` > [if the computer is white: `MOVE_REQUEST`, wait for `OnMoveComputerMsg`, send it as a move event] > `TURN_START` > user move message > (`MOVE_INVALID` or relay to engine with `MOVE_DATA`) > `TURN_END` > `MOVE_REQUEST` > `OnMoveComputerMsg` > move event > `TURN_END` > `TURN_START` and so on.
- *Two players.* each client sends `OnConnectMsg` and, after the tutorial, `OnStartGameMsg`; the server sends `WAITING` to the first and `INIT` to both once both have started; each move message is validated, relayed to the other client as a move event, and bracketed by `TURN_START` and `TURN_END` for both.
- *End.* `CHECKMATE` (and `CHECK`) events, then `OnGameWonMsg` to the winner and `OnGameLostMsg` to the loser; each client answers with `FRServer_GameOver`; the framework then runs its own end flow (game-over, reward status and score packets, see [[Minigame Framework]]); `FRServer_GameClose` and the Lua leave follow.

### What the Rust server must do

Items marked (R) are reconstructed policy, not original behaviour.

1. **Launch through the minigame framework.** The 12 type 6 rows are client data (`MiniGameData.txt`), so the server's job is to unlock them and start them through the normal framework flow (see [[Minigame Framework]]) with the row ids above, and to make sure the client has the `game_chess` movie, the tutorial movie and their sounds and icons. How the original told the client which movie to open is unknown.
2. **Speak the payload protocol.** Accept family 39 sub-opcode 14 payloads, treat the body as a NUL-terminated tab-separated string, and answer with the same packet type carrying a string that starts with a function name. Keep every string short (under 128 characters). Use the correct minigame state id in the header. Expect the handshake `OnConnectMsg`, then (after your `OnServerReadyMsg` and the tutorial) `OnStartGameMsg 0`; answer that with `INIT`.
3. **Run the chess rules yourself.** Track the position in the shared frame (a8 = 0), validate every move completely (legal for the side to move, no self-check, castling rights and path, en passant only on the immediately following turn, promotion piece 2 to 5), and detect check, checkmate, stalemate and draws (the client engine's own draw rule is two earlier matches of the position, 100 plies without a capture or pawn move, or insufficient material; which rules the original server applied is unknown, R). The client engine does not report anything and its `validate` allows self-check, so it can never act as the referee.
4. **Never let the engine see an illegal move.** Relay a move with `MOVE_DATA` only after full validation: the native handler drops a rejected move silently, which would desynchronise the computer's position from the server's. Use the byte encodings in the Data table; the executable does not convert them for you.
5. **One-player flow.** Send `LOCAL_AI` before the first turn (R for the exact argument, see Open questions), send `MOVE_REQUEST` with a `skill` for each computer turn, and expect `OnMoveComputerMsg` with four engine bytes. Treat an all-zero reply as "no move found" and decide the result yourself. Validate the proposed move on the server before showing it; do not trust the client. A server can also skip the client engine entirely and send move events for a server-side computer player (R): the movie treats a network move the same either way.
6. **Difficulty (R).** The original `skill` per row is unknown. Map Easy, Medium and Hard (rows with `DIFFICULTY` 1, 3, 5) to increasing values in the 0 to 100 range, where 100 or more disables random captures, and tune by playing the real client. If the client engine's search proves too shallow (see Open questions), prefer a server-side engine.
7. **Two-player flow.** Pair players for rows 65, 546 and 550, send `WAITING` to the first, `INIT` with the equal-arguments rule above to both, and relay moves with the correct conversion for a black local player (the movie flips indices itself, so send the white-at-bottom frame to both). Handle disconnects with `PLAYERLEFT` to the survivor. Pairing, invitations and timeouts are framework work.
8. **Turn events.** For each turn send `TURN_START` to both clients and `TURN_END` after the move is relayed; relay moves as events 10, 11, 12 or 13 to the waiting client only (the mover's client has already drawn the move and ignores move events in the user state). The order of `CHECK` and `CHECKMATE` relative to `TURN_END` is not recovered (see Open questions).
9. **Results and end of game (R).** On mate send `CHECKMATE`, then `OnGameWonMsg` to the winner and `OnGameLostMsg` to the loser; wait for each client's `FRServer_GameOver`, then run the framework's game-over and reward flow. For a draw or stalemate no event or message exists that shows a result: `OnGameStoppedMsg` ends the movie but the client then sends nothing, so the server must close the session itself through the framework (R). For a player who leaves, award the survivor whatever the framework's leave rules say; the original result is unknown.
10. **Rewards and goals (R).** Chess reward amounts, stars and leaderboard behaviour are not recovered; `LEADERBOARD_ID` is 0. Grant the "Chess Master" achievement on a win and count wins for the goal texts, using content-defined amounts; the numbers in the vault are not original. Quests that require a chess win or "beat singleplayer chess ten times" need the server to count wins per row family.
11. **Teardown.** The client frees its engine only when a new one is created (`DestroyChessInstance` has no caller, so it is probably the process-exit path); no per-game teardown is needed on the wire. Close the session on `FRServer_GameClose`, on the framework leave, or on disconnect.
12. **Do not rely on the book or time options.** The opening book is dead, and the engine's time parameters are never set; neither can be configured from the server.

### Evidence

- **Native map.** Four shards cover the chess engine: S0383 (the Lua bindings and copy/ctor helpers, 15 chess functions), S0527 (36, movegen part 1), S0528 (43, movegen part 2, init, evaluation, hash keys) and S0529 (27 chess functions, search, table, book). All 121 chess records are accepted. The generated table below lists 42 of them (S0383 and S0529); the S0527 and S0528 rows will appear when the table is regenerated. S0527 and S0528 were cross-reviewed by an independent model and carry a correction note at the end of each findings file; S0529 had a round 2 correction list.
- **Gaps in the map.** Shard S0526 (`00f48632..00f4cbe0`, 68 functions: `make_move`, `undo_move`, the `w_*` generators, `start`, the destructor, the API wrappers, `random_32`) failed and is not accepted. This pass read its small functions directly from the pseudocode (`start`, `is_computer_move`, `resign`, `get_winner`, the `convert_*` family, `random_32`, `ChessPosition`); `make_move` and `undo_move` (about 6.5 KB of machine code) were not read.
- **Movie.** The 103 ActionScript files exported by a previous worker from `game_chess.gfx` are a complete source for the client protocol and the turn state machine; they come from the shipped client, not from a server.
- **Video.** 128 frames in 47 videos mention the chess table. Every one is a world frame showing the purple "Checkers & Chess" marker with a distance (eight in Merry Vale, four in Shrouded Glade, three in Sweetwater Climb, two each in Seaside and Blackspore, one at the Sasparilla Brewery, the rest with no zone). **No video frame shows a chess board, a result or a reward**, and no quest frame references a chess reward.
- **Wiki.** The Board Games page (2013), the Achievements page ("Chess Master", "Win a game of Chess", bronze trophy, 5 points) and town pages that list the table. It has no chess rules, rewards or AI detail.
- **Vault.** `Research/Pieces/008 Chess.md` for the movie protocol table, the string ids and the native status mapping (checked against the movie source in this pass; one error found, the board origin). `Research/Original Activity Coverage.md` for the "no receipt" status.
- **Not used.** The community packet catalogue, any emulator or legacy-server code, and the project's earlier Rust session model.

### Open questions and contradictions

- **What `start(int)` means.** `ChessInit(computer)` stores its argument at `this+0x13`. `is_computer_move` is true when the side to move equals it (so it is the computer's colour), but `resign` writes the result `this[0x13] + 4` and `make_computer_move`, when it has no move, writes `3 - this[0x13]`, which fit the argument being the human's colour. Which value the server must send in `LOCAL_AI` for a computer playing black needs a native run. *client (S0526)*
- **Engine strength.** The time parameters are never written, so the search may be extremely shallow (R inference above); this needs a real-client test (play a game against each `skill`) before relying on the client engine. Also unknown: the `skill` values for the three difficulties and what "board 1, 2, 3" selects (the movie fixes the art set at 0 and the backdrop at frame 1).
- **Event order for check and mate.** The movie plays `INCHECK_ME` in the user state and `INCHECK_OTHER` in the network state, but `MATE_ME` in the network state and `MATE_OTHER` in the user state. Whether `CHECK` and `CHECKMATE` arrive before or after `TURN_END`, and whether both clients get them, cannot be settled from the movie.
- **Draws and stalemate.** No event, no resign and no clock exist; how the original ended a stalemated, repeated or abandoned game is unknown.
- **`INIT` arguments.** Only equality is tested; what the two numbers are (player ids?) is unknown.
- **Pairing and the table.** How a one-player row is chosen at the table, how two players are matched, why `MIN_PLAYERS` and `MAX_PLAYERS` are 0, and which of `game_chess.gfx` and `game_chess.swf` is loaded are framework questions; `Original Activity Coverage` records no captured receipt for any of the 12 rows.
- **Rewards.** No amounts, stars, leaderboard or end-screen data for chess; which quests use which row.
- **Move encoding gap.** The `0x80` double-push bit and the en passant value come from `convert_player_move` (unaccepted S0526) and the generators; a native run should confirm that the engine accepts exactly these bytes when fed through `MOVE_DATA`.
- **Contradictions carried forward.** Vault note 008 puts index 0 at a1 (wrong; a8). The S0527 findings said white is positive and the S0528 findings said side 0 is negative; the lead resolved this as white positive, side 0. The S0528 `in_check` record described the tested king and attackers with the sides inverted (the corrected reading: argument 1 tests the black king against white attackers). S0527's remark that `perft` skips promotion captures is wrong. S0528 left the writer of `DAT_01ca2a08` open; it is `search` (`00f57e10`). The three `does` texts for `history_sort_moves`, `stuff_pvs_to_tt` and `init_tt` in S0529 carry the round-2 corrections (three passes, up to `param_3 + 1` plies, `floor(log2)`). `ReadBookRecords` was `speculative` and the review argues `likely`. `ChessMove` default constructor `00c07f10` does not write byte 0.
- **Unknowns in the engine.** The 0x20-byte records at `+0x5e0`, who reads the pawn table `DAT_01caf548`, the rotation tables at `018727a8..01872fa8`, and the exact pawn-structure and king-safety constants of the evaluators.

### Key functions

| Address | Role |
|---|---|
| `00c080b0` | Lua `ChessInit`; calls `00c08010` (likely) |
| `00c08010` | `Chess_CreateAndStart`: frees the old engine, allocates 0x1290, constructs with a null book path, `start` (confirmed) |
| `00c080f0` | Lua `ChessMovePlayer`: four arguments, `validate`, then `make_player_move` (confirmed) |
| `00c081a0` | Lua `ChessMoveRequest(skill)`: `make_computer_move`, pushes four bytes (confirmed) |
| `00c08480` | Lua `SendMessage`: sends a movie string as a payload packet (label unknown, code read) |
| `009be6b0` | `SendMiniGameFlashPayload`: builds family 39 sub-opcode 14 (confirmed) |
| `009b89e0` | Receives a payload packet and raises `MiniGameFlash:OnMessage` (label unknown, code read) |
| `00f4eb70` | `Chess::Chess`: initialises tables, book handle and the start position (confirmed) |
| `00f53020` | `init_game`: start position, rotated occupancy, hash key (likely) |
| `00f53590` | `init_const_boards`: attack, between, ray, key and class tables (confirmed) |
| `00f50f10` | `validate`: pseudo-legal check of a human move (likely) |
| `00f4ec90` | `make_player_move` (confirmed) |
| `00f507c0` | `make_computer_move`: draw check, random capture, book, search (confirmed) |
| `00f506b0` | `get_random_capture` (confirmed) |
| `00f4eea0` | `valid_moves`: legal list, reversed, promotions doubled (likely) |
| `00f4e100` | `check_evasions` (confirmed) |
| `00f51590` | `in_check` (likely, side labels corrected) |
| `00f514b0` | `check_draw` (confirmed) |
| `00f57e10` | `search`: iterative deepening driver (confirmed) |
| `00f56eb0` | `negamax` (confirmed) |
| `00f58840` | `quiescent` (confirmed) |
| `00f58490` | `SEE`, `00f58750` `see_sort`, `00f582d0` `mvv_lva_sort`, `00f58220` `history_sort_moves` (unknown labels, code read) |
| `00f54a10` | `score`: phase dispatch (confirmed); `00f54410` `opn_score`, `00f53e00` `mid_score`, `00f539d0` `end_score` (likely) |
| `00f56ca0` | `add_move_to_tt`; `00f56ba0` `init_tt`; `00f56d20` `stuff_pvs_to_tt` (unknown labels) |
| `00f535e0` | `init_zobrist`; `00f53840` `gen_zob_key`; `00f53780` `gen_pawn_key` (likely) |
| `00f56830` | `use_book` (confirmed, dead in this client); `00f567e0` `init_book` (confirmed) |
| `00f50900` | `init_val_table`: move-class lookup used by `validate` (confirmed) |
| `00f4efe0` | `perft` (likely) |
| `00f50630`, `00f50740` | `mate`, `is_stalemate`: set the result code, no callers (confirmed) |
| `00f49170`, `00f492c0`, `00f49550`, `00f49330`, `00f49480` | `start`, `is_computer_move`, `resign`, `get_winner`, `convert_player_move` (client (S0526), unreviewed) |
<!-- NARRATIVE END -->

## Mapped functions (generated)

42 accepted functions: 26 confirmed, 6 likely, 1 speculative, 9 unknown. Network-linked (`wire`): 0.

Subsystems: `minigame.chess` (15), `minigame.chess.ordering` (7), `minigame.chess.search` (6), `minigame.chess.book` (4), `minigame.chess.tt` (4), `minigame.chess.movegen` (3), `minigame.chess.see` (2), `minigame.chess.debug` (1)

### `minigame.chess`

**Chess** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c07fa0` | Chess::Assign | confirmed | Copy assignment (operator=) for the whole Chess engine instance: memcpy(this, source, 0x1290) and returns this. |
| `00c07fc0` | Chess_DefaultCtorClosure | confirmed | Compiler-generated default-constructor closure: calls Chess::Chess(this, 0) (FUN_00f4eb70) with a null computer side and returns. |
| `00c08010` | Chess_CreateAndStart | confirmed | Destroys any existing Chess instance stored at *param_1 (Chess::~Chess + _free), allocates a fresh 0x1290-byte block (FUN_00d833a6), constructs Chess on it (FU… |
| `00c07ff0` | Chess_DeletingDtor | likely | Deleting destructor: calls Chess::~Chess (FUN_00f49190) on the object and then releases it with _free when bit 0 of the flag byte argument is set, returning th… |

**ChessControls** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c07f80` | ChessControls::Assign | likely | Copy assignment (operator=) for ChessControls: copies 0xf dwords (60 bytes) from the source and returns the destination (rep movsd). |

**ChessMove** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c07ef0` | ChessMove::ChessMove | confirmed | Four-argument ChessMove constructor: stores the four byte arguments into this[0..3] (from, to, promote, special) and returns. |
| `00c07f20` | ChessMove::Compare | confirmed | Compares two ChessMove values byte by byte (all four bytes must be equal) and returns true when they match, false otherwise. |
| `00c07f10` | — | unknown | Default ChessMove constructor: writes zero into all four bytes of the move (no arguments) and returns. |
| `00c07f50` | — | unknown | Copy assignment (operator=) for the 4-byte ChessMove: copies the packed dword from the source into the destination and returns the destination. |

**ChessMoveData** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c07f60` | ChessMoveData::Assign | confirmed | Copy assignment (operator=) for ChessMoveData: copies 8 dwords (32 bytes) from the source and returns the destination (rep movsd). |

**ChessPosition** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c07ed0` | ChessPosition::Assign | likely | Copy assignment (operator=) for ChessPosition: copies 0x76 dwords (472 bytes) from the source object into the destination (rep movsd) and returns the destinati… |

**(no class)** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00c080f0` | ChessMovePlayer | confirmed | Lua 'ChessMovePlayer' handler: with more than 3 arguments it reads from, to, promote and special (FUN_00e3a770 + FUN_00d83c90 each), builds a ChessMove, calls … |
| `00c081a0` | ChessMoveRequest | confirmed | Lua 'ChessMoveRequest' handler: with at least one argument it calls Chess::make_computer_move on the global Chess (DAT_01bec3b0) into a four-byte local, pushes… |
| `00c080b0` | ChessInit | likely | Lua 'ChessInit' handler: with at least one argument it reads the top value as an integer (FUN_00e3a770 + FUN_00d83c90) and calls the create-and-start helper 00… |
| `00c082a0` | DestroyChessInstance | likely | If the Chess pointer stored at *param_1 is non-null, calls Chess::~Chess on it (FUN_00f49190), frees the block (_free) and nulls the pointer. |

### `minigame.chess.book`

**Chess** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f567e0` | Chess::init_book | confirmed | Closes the FILE* at this+0x1288 if one is open and clears the slot. Returns false for a null path; otherwise calls fopen(path,"rb"), stores the handle at this+… |
| `00f56830` | Chess::use_book | confirmed | Opening-book probe. Returns 0 when this+0x1e8 > 3 or no book FILE* is open. Builds the position key from this+0/this+4 (XOR with change_side_zob when this+0x10… |

**(no class)** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f566a0` | ReadUint32 | likely | Reads 4 bytes through FUN_00d8ae0b (fread wrapper) from the stream in EAX into a local, then stores the value as a little-endian dword at *param_1. The byte-by… |
| `00f566e0` | ReadBookRecords | speculative | Takes the record count in ECX, the destination in EDX and the stream in EDI (not set here). For each record it reads 20 bytes with FUN_00d8ae0b, stores bytes 0… |

### `minigame.chess.debug`

**Chess** (1)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f56de0` | Chess::print_pvs | confirmed | For a ChessLine (count at +0, moves from +4) prints each move via move2algebraic and wprintf: the first move with the format "PVS-%s", the rest with a second f… |

### `minigame.chess.movegen`

**Chess** (3)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f55c40` | Chess::gen_moves | confirmed | Returns a 64-bit destination mask in EDX:EAX. Param_1 selects one of two piece-set branches; for each set bit of the piece bitboards (this+0x148..0x19c) it cal… |
| `00f563b0` | Chess::gen_attackers | confirmed | Returns a 64-bit set in EDX:EAX of squares attacking the square/side given by param_1, combining per-square attack lookups (pawn, knight, king and slider table… |
| `00f55250` | — | unknown | Returns a 64-bit square mask in EDX:EAX. Param_1 selects one of two piece-set branches (bitboards at this+0x158..0x164 or this+0x188..0x19c); for each set bit … |

### `minigame.chess.ordering`

**Chess** (7)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f581a0` | Chess::add_move_to_hh | confirmed | If the stop flag this+0x1264 is not 1, adds depth*depth (the second argument squared) to the history counter at index byte0*64 + byte1 of the move in DAT_01caf… |
| `00f581e0` | Chess::score_move | confirmed | Returns the history counter at index byte0*64 + byte1 of the ChessMove at param_1 (one stack argument, ret 4). |
| `00f58200` | Chess::init_history_heuristics | confirmed | Zeroes the 0x1000 history dwords at DAT_01cafb98 (rep stosd). |
| `00f582a0` | Chess::shift_history | confirmed | Arithmetic-shifts every history dword from DAT_01cafb98 up to DAT_01cb3b98 (4096 entries) right by 2, i.e. divides each by 4. |
| `00f58220` | — | unknown | Bubble-sorts n ChessMove records (4-byte moves) in place, swapping an adjacent pair when the next move's history counter is smaller than the current one, so th… |
| `00f582d0` | — | unknown | Sorts a ChessMove range in place by adjacent pairwise swaps. The ordering key is the difference of piece values (table DAT_01b964dc, indexed by board codes at … |
| `00f58750` | — | unknown | For n capture moves, scores each one: 0 when flag bit 4 is set; the promotion piece value when the promote byte is nonzero; otherwise SEE when the victim value… |

### `minigame.chess.search`

**Chess** (6)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f56b10` | Chess::end_search | confirmed | Sets the search stop flag this+0x1264 to 1 and returns; no other effect. |
| `00f56b20` | Chess::check_time | confirmed | Reads the time with __ftime64_s and, when the current time (seconds plus millitm scaled) exceeds the deadline _DAT_01caf538 + _DAT_01caeef0 (search start plus … |
| `00f56b70` | Chess::clear_killers | confirmed | Zeroes two 800-byte tables with memset: DAT_01caf550 and DAT_01caeef8 (two killer slots per ply, plus a count table). |
| `00f56eb0` | Chess::negamax | confirmed | Fail-soft principal-variation negamax over (depth*60 remaining, window param_2..param_3, PV ChessLine param_5). Per node: time check every 0x20000 nodes, check… |
| `00f57e10` | Chess::search | confirmed | Iterative-deepening driver. Resets the node counters (this+0x1270..0x1278), sets the depth counter this+0x126c to 1, clears the stop flag and both killer table… |
| `00f58840` | Chess::quiescent | confirmed | Quiescence search. Returns the stand-pat score (score) when it cuts; returns param_2 when the depth counter param_4 exceeds 0x41. When in check it searches che… |

### `minigame.chess.see`

**Chess** (2)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f583b0` | Chess::get_xray | confirmed | Returns the 64-bit set of squares reached by the sliding ray through (param_1, param_2) that is revealed behind the piece there. The ray type comes from the pi… |
| `00f58490` | — | unknown | Static exchange evaluation for the capture sequence on squares (param_1, param_2). Starts from the attacker set of the side in param_2 (gen_attackers) with x-r… |

### `minigame.chess.tt`

**Chess** (4)

| Address | Name | Confidence | What it does |
|---|---|---|---|
| `00f56ca0` | Chess::add_move_to_tt | confirmed | Stores a 16-byte transposition entry in DAT_01caf544 at index (this key low dword & mask), with the position key (this+0 and this+4) at +8 and +0xc, the move a… |
| `00f56d90` | Chess::add_move_to_pt | confirmed | Writes a 16-byte record into DAT_01caf548 at index ((this+8) & _DAT_01c364e8): the value this+8 at +0, this+0xc at +4, and the int argument at +8. |
| `00f56ba0` | — | unknown | Stores the bit length of DAT_01b964f8 in DAT_01caf540, computes the entry count 1<<n with __allshl, mallocs two tables of 16-byte entries (DAT_01caf544, DAT_01… |
| `00f56d20` | — | unknown | Walks the principal-variation ChessLine (count at +0, moves from +4) up to param_3 plies: for each move calls add_move_to_tt(move, param_2, depth param_3 - ind… |

## Source shards

[[S0383 00c05780-00c0db00|S0383]] (15), [[S0529|S0529]] (27)
