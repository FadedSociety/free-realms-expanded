# Chess Service (W41)

Status: scoped `osfr-tools` implementation and focused tests. This note records client facts separately from reconstructed server policy. No original server source or chess packet capture exists.

## Recovered client facts

- `notes/Build Guides/Gameplay/Chess.md` is the reviewed specification. It records 12 type-6 Chess rows: nine single-player rows across Easy/Medium/Hard and three two-player rows (65, 546, 550); the other row IDs are 20, 66, 67, 543-545, and 547-549. The board uses a8=0 and side 0 is White. The client movie uses the event codes and text messages listed below.
- The movie's computer engine proposes moves. `ChessMoveRequest(skill)` calls the native computer-move routine and returns four bytes to Lua; the movie sends those as `OnMoveComputerMsg`. The server must validate this proposal itself before relaying it. Native function `00c081a0` extracts the supplied skill, calls `00f507c0` (`Chess::make_computer_move`, accepted/confirmed), and returns four move bytes. This is client engine behavior, not proof of any original server's move validation.
- Native `00f49480` writes `from`, `to`, zero promotion, and a special byte, then sets the special bits based on occupancy and piece/move shape. It confirms double-push `0x80`, en-passant `0x64`, capture `0x60`, and the white castle flags (destination 62->2, 58->1). Its black-side branches use destination 6->16 and destination 1->8, which conflicts with the reviewed Chess guide's standard c8 destination 2 for black queenside castling. Because this S0526 reading is marked unreviewed and the guide is the reviewed specification, Rust uses destination 2->8. Castle requests use the engine sentinel `[0, 0x3e, 0, flag]`. The older `piece_008.rs` candidate reverses the white castle flags in `special_code`, so that code is not used for this service's wire encoder.
- Native `009be6b0` calls `MiniGamePayloadPacket::CtorWithPayload` at `009b09d0` to send a movie string through the minigame payload path. The Chess guide identifies family 39, sub-opcode 14. The packet encoder here writes the 19-byte header, a UTF-8 string, and a trailing NUL; the signed payload length includes that NUL. Header identity is the table's instance ID, group `-1`, and Chess movie ID 20.
- The guide specifies event IDs: INIT 0, TURN_START 1, TURN_END 2, MOVE 10, CASTLE 11, PROMOTE 12, EN_PASSANT 13, CHECK 14, CHECKMATE 15, MOVE_INVALID 20, MOVE_REQUEST 30, MOVE_DATA 31, PLAYERLEFT 40, WAITING 41, LOCAL_AI 42. The movie message names include `OnConnectMsg`, `OnStartGameMsg`, `OnMoveMsg`, `OnCastleMsg`, `OnPromoteMsg`, `OnEnPassantMsg`, `OnMoveComputerMsg`, `FRServer_GameOver`, `FRServer_GameClose`, `OnGameWonMsg`, `OnGameLostMsg`, and `OnGameStoppedMsg`.
- The game guide says the client engine checks for repetition (two earlier matches), 100 plies without a capture or pawn move, and insufficient material (`00f514b0`, `Chess::check_draw`, accepted/confirmed in its generated map). It identifies stalemate/draw display behavior as unresolved: the movie has no draw/stalemate event handler, and `OnGameStoppedMsg` ends the movie without a result acknowledgement.

### Native-map commands and provenance

Commands run from the project root:

```text
python -I scripts/native_map/nmq.py dis 00f49480 256
python -I scripts/native_map/nmq.py dis 00c081a0 256
python -I scripts/native_map/nmq.py dis 009be6b0 256
python -I scripts/native_map/nmq.py dis 00f514b0 256
```

The first output contains the encoder stores and branches at `00f49493..00f49543`. The second output shows `00c081d4..00c08243` passing the requested value to `00f507c0` and pushing the four returned bytes. The third output shows `009be71c..009be721` invoking `009b09d0` before queuing the string-body packet. The fourth output shows `00f514b0..00f51569` traversing stored position keys and checking the draw counters/material fields, consistent with the guide's `check_draw` description. These are client code observations; server behavior below is not recovered from them.

## Server implementation and reconstructed policy (R)

- `src/chess.rs` owns a position per table, validates every human move and client-engine proposal against legal moves, and handles castling, en passant, all four promotions, check/mate, stalemate, repetition, insufficient material, and a 100-halfmove draw threshold. The 100-halfmove rule is reconstructed; the client guide attributes it to the native engine, but no original server rule was found.
- The existing `src/restoration/piece_008.rs` candidate was not reused because its `apply_unchecked` clears the destination after placing a capturing mover there. A focused Kiwipete depth-2 count reproduced 2053 instead of the standard 2039. The owned engine clears the captured square before placing the mover; the candidate file was left untouched.
- `Mode::for_row` maps the nine one-player rows and three two-player rows from the guide. Easy/Medium/Hard map to engine skills 20/60/100 (R). The one-player seat color is selected by the caller. INIT values `(1,1)` for white and `(1,0)` for black are R; the movie uses equality between the pair to determine the local side, but their meaning is unknown. LOCAL_AI names the computer side.
- In solo play, a legal human move is sent to the client engine as MOVE_DATA, then the service requests the computer proposal with MOVE_REQUEST and the row's skill. The service rejects an illegal engine proposal without changing the board and requests another. It validates and applies a legal proposal before emitting the move event.
- In two-player play, both clients receive INIT and turn events; the opponent alone receives the validated move event. Player IDs are attached to the per-player instance IDs when building family 39/14 payloads. A leaving player's survivor receives PLAYERLEFT.
- A decisive result emits CHECK when applicable, then CHECKMATE, then TURN_END, followed by `OnGameWonMsg`/`OnGameLostMsg`. The relative CHECK/CHECKMATE versus TURN_END order is unresolved by the movie and this ordering is R. A first `FRServer_GameOver` from each participant unlocks that participant's framework settlement; duplicate acknowledgements are ignored. Draw/stalemate sends `OnGameStoppedMsg` and is settled by the outer coordinator because the client sends no game-over acknowledgement for it; the shared lifecycle represents both draw seats as Lost because its `ActivityFinish` has only a won boolean (R).
- Resignation and disconnect are explicit service outcomes (R). On a two-player disconnect, the survivor receives PLAYERLEFT and the disconnected seat is not sent a result message; the disconnected player's framework settlement is immediately ready because that client cannot acknowledge. Resignation and a computer no-move result send the appropriate win/loss movie messages.
- `settle_player` delegates to the existing `MinigameRuns::finish` path and accepts framework score, stars, elapsed time, and an optional reward grant from its caller. It awards a grant only to the recorded winner and deduplicates by player/table; the shared run lifecycle also rejects a repeated terminal finish. Reward amounts, score/stars, and leaderboard behavior are not recovered here.

## Verification

`tests/chess.rs` covers perft vectors: start position (20, 400, 8,902, 197,281), Kiwipete (48, 2,039, 97,862), and promotion position 5 (44, 1,486, 62,379). These expected counts are from the standard [ChessProgramming.org perft results](https://www.chessprogramming.org/Perft_Results). The suite also covers pinned/illegal moves, a discovered check, Fool's Mate event order and once-only framework reward settlement, solo proposal validation and request flow, two-player relay, threefold-draw movie shutdown and both lifecycle results, resignation, disconnect forfeit, draw/stalemate/repetition policy, payload bytes, row skills, and castle encoding.

Build/test command (private toolchain and task target):

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W41'
..\..\scripts\cargo.ps1 test --test chess
```

The focused command completed successfully: **12 passed, 0 failed** in 1.06 seconds. It compiled the current shared crate with one unrelated `src/housing.rs:567` dead-code warning. An earlier diagnostic run showed the 008 candidate also returns 2053 for Kiwipete depth 2; that exploratory comparison was removed from the final test suite after the candidate bug was found.

## Open gaps

- No original chess server source, server packet capture, or native-client chess playthrough confirms pairing, INIT values, engine skill values, event ordering, move relay acceptance, reward contents, or disconnect/resign outcomes.
- Native `00f49480` is from the map's unreviewed S0526 reading; its black queenside branch conflicts with the reviewed guide. Confirm all castle move bytes against a native client before treating the event bit mapping as runtime-accepted behavior.
- Difficulty values and the 100-halfmove draw rule remain reconstructed. Draw/stalemate teardown must be exercised against the real client because it sends no `FRServer_GameOver` response.
- `src/chess.rs` is not registered in shared `lib.rs` under this task's ownership boundary. The lead must review and integrate that registration.
- Pairing/timeouts, reconnect behavior, production save transaction boundaries, content-sourced rewards, and quest/leaderboard updates remain integration work.
