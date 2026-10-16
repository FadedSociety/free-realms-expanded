# Candidate Evidence Reconciliation

Date: 2026-10-13  
Task: W07, Build Ledger item 1  
Status: Chess and Forging candidate claims corrected; both candidates remain unregistered.

## Changes

- `prototype/osfr-tools/src/restoration/piece_008.rs` now stores the board with client index 0 at a8 and index 63 at h1. FEN parsing/output, pawn direction, promotion, castling, en passant and rook-right updates use that frame. Session move records use the guide's four-byte engine layout, including the castle sentinel `[0, 0x3e, 0, flag]`.
- `prototype/osfr-tools/tests/piece_008.rs` now tests a8/a1 and king indices, FEN round-trip, special bytes and exact ordinary/castle move records. The new coordinate expectation failed on the former model before the source change.
- `prototype/osfr-tools/src/restoration/piece_018.rs` describes Forging game 139 as `TYPE 8 MatchThree` and removes the stale TYPE 9 claim.
- `prototype/osfr-tools/tests/piece_018.rs` explicitly binds route 139 to type 8 and the activity catalogue.
- `prototype/osfr-tools/src/derby.rs` was not changed. Its packet lengths and subtype names remain quarantined.

## Recovered client evidence and limits

### Chess

Recovered facts used: the movie, wire and engine use a8 = 0; white's king starts at 60 and black's at 4; the second player's movie view applies `63 - index`. The move table specifies `[from, to, 0, flags]` for normal moves, `[0, 0x3e, 0, flag]` for castling, promotion special `0x40` or `0x60`, and en passant `0x64`. The guide identifies double-push `0x80`, occupied-target `0x60`, promotion `0x40`, and castle flags 1/2/8/0x10.

Provenance: [[Build Guides/Gameplay/Chess#Coordinates and conversion]] (project guide lines 111–120) synthesizes the shipped `game_chess.gfx` ActionScript, including `GameState.as` and the network/event classes. The exported movie is preserved at `../lab/reports/piece-008-20261009T024347Z/chess-movie/scripts/`. The guide attributes conversion to client pseudocode functions `convert_player_move`, `convert_player_promote`, `convert_player_castle` and `convert_castle_to_king`; its key-functions table identifies `convert_player_move` at `00f49480`.

Limit: the guide records shard S0526 (`00f48632..00f4cbe0`) as failed and unaccepted. It says the small conversion functions were read directly from pseudocode, and specifically leaves native `MOVE_DATA` acceptance of `0x80` and `0x64` for a real-client check. These tests prove the candidate matches the guide; they do not prove engine acceptance or historical server behavior. The chess session and server rules remain reconstructed policy (R), and the candidate remains unregistered.

### Forging row 139

Recovered fact used: `MiniGameData.txt` row 139 is TYPE 8, NAME_ID 386081. The activity receipt review binds that source row to native entry/completion and records its result; it says the old TYPE 9 Mimic-anvil wording was wrong. The minigame guide identifies TYPE 8 as MatchThree, including Forging.

Provenance: [[Build Guides/Gameplay/Minigame Framework]] (TYPE registry at project guide line 167); [[Reviews/Activity Coverage Receipt Review]] (row correction at line 24, raw row and receipt binding at lines 28–36). The exact source row is `139^8^386081^22054^910^…` in the review. This establishes row 139 only. The original service engine class and the other 60 Forging rows remain unproven; the Rust route's implementation is not client evidence.

### Derby quarantine

`derby.rs` packet claims rely on a capture under `lab/references/OpenSourceFreeRealms`. The Demo Derby guide explicitly excludes that community capture and says its frame counts, lengths and values are not evidence for this protocol. Only layouts derived from native client readers and senders can establish the packet lengths and subtype meanings. Until then, keep those claims quarantined and leave `derby.rs` untouched.

Provenance: [[Build Guides/Gameplay/Demo Derby]] (“What this page does not use,” project guide line 16); [[Design/Build Ledger#Candidate claims contradicted by the current guides/reviews]] (line 23).

## Verification

- RED baseline, private Rust compiler, from `project/prototype/osfr-tools`:
  ```powershell
  $env:CARGO_HOME = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\cargo'
  $env:RUSTUP_HOME = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\tools\rustup'
  $env:PATH = "$env:CARGO_HOME\bin;$env:PATH"
  & "$env:CARGO_HOME\bin\rustc.exe" --edition=2024 --test tests/piece_008.rs -o '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W07\piece_008-red.exe'
  & '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W07\piece_008-red.exe'
  ```
  → 3 passed, 13 failed. The a8-origin check found a white rook at index 0 instead of the expected black rook; opening e2–e4 was rejected.
- Requested focused build, from `project/prototype/osfr-tools`, with `CARGO_TARGET_DIR=%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W07`:
  ```powershell
  $env:CARGO_TARGET_DIR = '%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\W07'
  & '..\..\scripts\cargo.ps1' test --test piece_008 --test piece_018
  ```
  → `piece_008`: 17 passed; `piece_018`: 12 passed, 2 ignored for private opening content.
- An earlier run of that Cargo command stopped before the target tests at unrelated `src/character_store.rs:106` (`fs::rename` did not satisfy the `FnMut`/`FnOnce` bounds). A later rerun of the same command completed successfully; no shared source was changed for W07.
- Private `rustfmt.exe --edition 2024 --check` on the four owned Rust files → pass.

## Open gaps

- No chess client was launched. Native engine acceptance of the move bytes, especially `0x80` and `0x64`, remains open per the Chess guide.
- The corrected chess and Forging modules are candidates, not registered or routed features. Forging evidence covers row 139, not the full TYPE 8 family.
- Derby packet layouts remain unknown until each claim is grounded in native client reader/sender code.
