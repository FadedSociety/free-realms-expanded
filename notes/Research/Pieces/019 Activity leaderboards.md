# 019 Activity leaderboards

Queue item 019. Acceptance: recover score direction/ties/reward rules and keep
persisted records from trusting arbitrary client scores.

Status 2026-10-09 02:50 UTC: evidence recovered and a bounded server-side store
implemented with 8 Rust tests and 10 Python checker tests passing. No native or
gameplay acceptance was attempted; the original leaderboard service is not
preserved, so direction, ties and rewards for the mini-game boards stay split
between recovered facts, inferred meaning and reconstructed policy.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_019.rs` | Leaderboard store, client row payload builder, snapshot persistence. Owned module, not registered in `lib.rs` |
| `prototype/osfr-tools/tests/piece_019.rs` | 8 focused tests; includes the module by `#[path]` |
| `scripts/pieces/piece_019.py` | Read-only evidence extractor over the preserved lab corpus |
| `tests/pieces/test_piece_019.py` | 10 stdlib tests (fixture parsing, real-source pins, CLI determinism) |
| `../lab/reports/piece-019-20261009T025056Z/` | FFDec exports of two shipped movies plus `evidence.json`, `provenance.json`, `run.log` |

## Recovered original facts

1. **The in-game leaderboard is a web fetch, not a game packet.** The client
   reads `GameLeaderBoardUrl` from its configuration and appends the request to
   `http://www.freerealms.com/ingameleaderboard/leaderboardScores.action?`
   (`ClientConfig.ini:40`, `ClientConfig_stage.ini:37`). `FUN_00b85d50` loads
   the key into a `StringFixed<512>`; `ClientLeaderBoardManager::RequestLeaderBoard`
   logs `sending: %s`, throttles with a 1000 ms timer, and
   `ClientLeaderBoardManager::OnGetResult` parses the reply and looks up the
   `result` node (`pseudocode.c` 1557796, 1557842-1557849, 1558301-1558344).
   A separate `NetworkCommand_LeaderBoardData` command id `0x1ca` exists in the
   client (3375015/3375024, 3375033).
2. **Which games have boards.** `MiniGameData.txt` declares 14 caret columns and
   783 game rows; 772 rows carry `LEADERBOARD_ID=0`. Exactly 11 rows carry a
   board id, and all 11 are TYPE 8 match-three boards:

| Game | Name | LEADERBOARD_ID |
| --- | --- | --- |
| 715 | Copper Mining | 91 |
| 716 | Tin Mining | 93 |
| 717 | Iron Mining | 95 |
| 718 | Silver Mining | 97 |
| 719 | Gold Mining | 99 |
| 720 | Queensfields Harvesting | 109 |
| 721 | Farnum's Farm Harvesting | 111 |
| 722 | Chugaroot Farm Harvesting | 113 |
| 723 | Perry's Pastures Harvesting | 115 |
| 724 | Lakesfield Harvesting | 117 |
| 725 | Thorny Tillage Harvesting | 119 |

   Names resolve through the community locale export (`lab/reports/locale-id-to-text.json`).
3. **Client row contract.** `MinigameStartScreen.gfx` (Assets_008.pack entry
   5942) was exported with the pinned FFDec 26.3.0 build. `ListPanel.dataRequest`
   sends `RequestData(offset, pageSize, mode, callback)` with mode 1 for
   high scores; `ItemDisplay.setData` parses rows separated by `##`, properties
   by `::`, and property values by `^`; `HighScoreItem` reads `Rank`, `Name`,
   `Guid`, `Value`, `Id`, `Type`, rendering `Type` 1 as a comma-grouped points
   score and `Type` 2 as elapsed seconds; the high-score mode is ten rows in one
   column.
4. **UI vocabulary.** Locale ids 430659/430660/430661/430662/430670/430671/430734/430735
   are `My Leaderboard`, `Leaderboard Name`, `Top Score`, `Character Name`,
   `Leaderboards`, `Select Leaderboard`, `My Rank`, `Change Leaderboard`; 409152
   is "Now you can view leaderboards in game!  Click on the High Scores button
   from the minigame start screen."
5. **End-score wire.** `MiniGameGameEndScorePacket` is packet 47 under
   `BaseMiniGamePacket = 39` (community packet catalogue, `PacketCodes.cs:23,663`)
   and the client constructor is `FUN_008a4420(0x2f, ...)` with a
   `SoeUtil::Array<MiniGameScoreInfo>` field (`pseudocode.c` 1200331-1200336).
   The per-entry field layout of `MiniGameScoreInfo` was not recovered.
6. **Client-side per-game state.** Debug-name tables list `Total Score`,
   `LeaderboardId` (`pseudocode.c` 1704764, 1704776) and, for the minigame
   browser, `Leaderboard Position` and `High Score` (1704708, 1704710).
7. **First-party tie/reward text exists only for TCG events.** Locale ids
   433371/433983 (and 433981-433997) carry the event rules: rankings are posted
   per event, most points wins, ties break on most games won, then most games
   played, then joint winners; prizes are awarded at event conclusion; no prizes
   are awarded for Top Duelist Games. This is event text, not the mini-game
   high-score board.

## Inferred meaning (not recovered)

- Points rank high to low: the value column is labelled `Top Score`, the client
  never re-sorts rows, and it renders the service-assigned `Rank` verbatim.
- Elapsed seconds rank low to high, following race-style timing and the client's
  own time formatter.
- `Rank` in the payload is produced by the (unpreserved) ranking service, not by
  the client.

## Reconstructed policy (explicitly not historical)

- A board entry is the character's single best result; a worse re-run is
  consumed but does not replace it.
- Ties share a competition rank (1, 1, 3).
- Values are bounded to `0..=9_999_999_999` because `HighScoreItem.setScore`
  writes only `txt0..txt9`.
- Rows are served ten at a time, and `page_payload` returns the total count the
  client needs for its scrollbar.
- A value enters a board only through a server session that was opened and
  finished by the server, commits once, and never accepts a negative or
  over-ceiling value. Character id/name must be non-empty and free of the three
  wire delimiters, so a name cannot inject extra rows or properties.
- Snapshots are JSON with a version field, written with the same
  same-directory temporary-file replacement discipline as `character_store`.

## Gaps

- The original web query parameters (`leaderboardScores.action?...`) are not
  recovered; only the base URL string is preserved.
- The original ranking service is not preserved, so the direction/tie rules
  above stay inferred or reconstructed rather than historical.
- No mini-game leaderboard reward rule is recovered. No reward is granted by
  this module.
- Time scoping hinted at by the `Most Recent` label (locale 430669) and any
  membership gating are unrecovered.

## Provenance (read-only inputs)

| Input | Bytes | SHA-256 |
| --- | --- | --- |
| `working/OSFR.Client/OSFR Client/Resources/MiniGameData.txt` | 30,734 | `8ce3cc61a2ad324912465e8d1785d246ccd680a01712e256fddf702ce856c252` |
| `working/OSFR.Client/OSFR Client/ClientConfig.ini` | 1,395 | `12ed501ce76d70b20a5c8ee7d4141e4431059a29e046bd63d9bd8ff199588cb5` |
| `working/OSFR.Client/OSFR Client/ClientConfig_stage.ini` | 1,304 | `5e91a68c0248966834863e2b0a3b86a79f9522e1d0c34a845eadd106f789a76f` |
| `decompiled/client/FreeRealms/pseudocode.c` | 95,176,801 | `f84a58674b8e7de0829d562e5f106a368f34d4750894a14b093ae745fa0befe3` |
| `references/OpenSourceFreeRealms/.../PacketCodes.cs` | 50,276 | `f3f1287671de00cd009dd86ceaeec4765be581cd823d1da3306910361ccd0869` |
| `reports/locale-id-to-text.json` | 6,788,031 | `f1f22a30107c824af4b3021c8c7c323c927d3c6a9b82efc387e9eaba6ae62844` |
| `extracted/client-assets/pack_20/005942.gfx` (MinigameStartScreen) | 40,141 | `c9a55a24d21f39871327342fd87d38c54da8363274a2d8dce9b7751fc615f684` |
| `extracted/client-assets/pack_19/006110.gfx` (scoreScreen) | 62,126 | `b26be1b239b0a35253e6b27da6fc9b2ff9eb07f3e1b2675ea98fb34256efdc8c` |

The movie exports come from the pinned community FFDec 26.3.0 under
`lab/tools`, run with the private JDK 21. The `scoreScreen.gfx` export is
supporting evidence only: it shows race/derby result rows (`Place`, `Guid`,
`Name`, `Best Time`, `Total Time`, `Finished Time`), which belong to the race
controllers rather than the web high-score board.

## Verification

```powershell
# RED 1: owned module did not exist yet
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_019
# error: couldn't find file `tests\..\src\restoration\piece_019.rs`

# RED 2: extractor did not exist yet
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_019.py" -v
# FileNotFoundError: ... scripts\pieces\piece_019.py

# GREEN
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_019
# 8 passed; 0 failed; 0 ignored

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces -p "test_piece_019.py" -v
# Ran 10 tests ... OK

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_019.py `
    --lab '..\lab' --out '..\lab\reports\piece-019-20261009T025056Z' --stamp '2026-10-09T02:50:56Z'
# {"boards": 11, "missing_inputs": [], "evidence_sha256": "0e7158525de56c1ec26cff48e72a6f9ce01e6a90d3d4a550a279cd0db5d66fd1"}
```

A second extractor run with the same inputs and stamp produced a byte-identical
`evidence.json`; the Rust tests additionally cover the client payload delimiters
by re-parsing the payload with the client's own split rules.

## Integration and native gates

- The module is not registered in shared `lib.rs`; the lead owns that change.
  `tests/piece_019.rs` includes it by `#[path]` so the checkout keeps building.
- No gateway, save file, content package, release binary or launcher was touched.
- Native acceptance was not attempted and is not claimed. The smallest future
  native check would be: serve the recovered row payload from a local responder
  bound to the configured leaderboard URL for a board such as Copper Mining 91,
  open the minigame start screen high scores, and confirm the ten-row panel and
  `My Rank` values. That requires the client to accept a redirected endpoint,
  which is unproven, and the original web service is unreachable by design.
- Wire integration is separate: matching `MiniGameGameEndScorePacket` (47) fields
  would need the `MiniGameScoreInfo` layout, which is still missing.

## Skills applied

`reverse-engineering` for the pseudocode/field-table recovery, `game-recon` for
the ForgeLight/Scaleform format and tool fingerprint, community-tool-first work
(pinned FFDec 26.3.0 export and the OpenSourceFreeRealms packet catalogue) with
RTK (normal commands first), Ponytail (reuse the existing export tooling, one
self-contained module, no new dependency) and Caveman (concise reporting).
`asset-pipeline`, `game-research-websearch` (no network access was used) and
`game-automation` were not needed.

Cross-links: [[../MMO Work Queue]], [[../Original Activity Coverage]],
[[../../WorkItems/019]].
