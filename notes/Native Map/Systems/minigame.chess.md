# System: minigame.chess

15 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (8)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c07ef0` | `ChessMove::ChessMove` | ChessMove | Constructs the 4-byte move value the chess engine validates and applies; mirrors the four flash arguments of ChessMovePlayer. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07f20` | `ChessMove::Compare` | ChessMove | Move equality test used by the chess engine (e.g. repetition/hash comparisons). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07f60` | `ChessMoveData::Assign` | ChessMoveData | Compiler-generated assignment of the engine's per-move context structure. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07fa0` | `Chess::Assign` | Chess | Compiler-generated assignment of the 0x1290-byte chess engine state. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07fc0` | `Chess_DefaultCtorClosure` | Chess | Used where the compiler needs a function pointer that default-constructs a Chess instance (the real constructor has a parameter, so MSVC emits this closure). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c08010` | `Chess_CreateAndStart` | Chess | Native implementation behind the MiniGameFlashC.ChessInit entry: (re)creates the chess engine and starts a game with the given computer side; note 008 describes this function as the ChessInit allocation + start. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c080f0` | `ChessMovePlayer` |  | Handler registered as 'ChessMovePlayer' in the MiniGameFlashC module (0x0182a1e8); note 008 recovers it as read four flash arguments into a ChessMove, Chess::validate, then Chess::make_player_move. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c081a0` | `ChessMoveRequest` |  | Handler registered as 'ChessMoveRequest' in the MiniGameFlashC module (0x0182a1e8): asks the local chess engine for the computer's move and hands the four bytes back to the movie; note 008 recovers the same contract. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## 🟢 likely (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c07ed0` | `ChessPosition::Assign ⚠downgrade` | ChessPosition | Compiler-generated assignment of the whole chess position (board array and counters) used when the engine keeps a position copy. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07f80` | `ChessControls::Assign ⚠downgrade` | ChessControls | Compiler-generated assignment of the board-control masks (castling/en-passant state) tracked by the engine. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07ff0` | `Chess_DeletingDtor → Chess::ScalarDeletingDtor` | Chess | MSVC deleting destructor for the heap-allocated Chess engine instance (paired with the create path 00c08010). | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c080b0` | `ChessInit` |  | Handler registered as 'ChessInit' in the MiniGameFlashC module (entry table 0x0182a1e8); ScriptsBase.lua declares MiniGameFlash.commands.ChessInit(computer) for it. Note 008 attributes the ChessInit entry to 0x00c08010;… | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c082a0` | `DestroyChessInstance` |  | Tears down the global chess engine instance (the counterpart of the create path 00c08010) when a chess game ends or is replaced. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c07f10` | `— ⚠reject` | ChessMove | Zero-initialises a move value before validate/make_player_move. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |
| `00c07f50` | `— ⚠reject` | ChessMove | Compiler-generated move assignment. | [[Native Map/Shards/S0383 00c05780-00c0db00|S0383]] |

