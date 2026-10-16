# Mining Pending Fall Reshuffle

Read-only diagnosis, 2026-10-09. Scope: run
`lab/runs/client-baseline-20261009T134930Z` (41 chains, 25,400 score, collected
`[0,0]`, 3 reshuffles). Owned here; no source, UI, save, process or release
change. Related: `Mining Mouse Ore Strategy.md` (driver efficiency, separate).

## Verdict first

**The live pending-fall race is NOT PROVEN - for the captured trigger it is
disproven.** The first reshuffle (save 78 -> 79) was the ordinary no-move
reshuffle over six *resting* (`idle=true`) ores; the real captured chain #19
advanced no ore and left nothing pending. Re-running the real pre-state and the
real payload through the public `Type8Engine` reproduces the real post-state
bit-for-bit. The crafted pending-bottom case remains only a bounded private
counterexample and is consistent with the original no-move reset ordering; no
source patch is authorized or indicated.

## Replay (public engine, not a port)

Private probe: `lab/reports/mining-pending-fall-reshuffle-20261009T140740Z/probe/tests/pending_fall.rs`
(sha256 `2e28374b165be6cb92c0b2ade0ce0fc2bf7817cba0fcf65a4f8c9ef66abb24bf`),
linked against production `src/mining.rs` sha256
`6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed`.
Command:

```
cargo test --manifest-path <probe>/Cargo.toml --test pending_fall -- --nocapture --test-threads=1
```

Result 6 passed / 0 failed (`logs/17-probe-run.log`, `logs/18-probe-green.log`):

- `captured_chain19_reproduces_save_079`: loading captured `state-078.json`
  (18,000,003 / Playing) and handling the exact wire payload
  `OnChainMsg\t10,3,10,4,10,5,9,5,` (trailing comma preserved) emits
  `OnLevelEventMsg 21 600 0`, `25 8 1`, then `11 1` (reshuffle) plus the
  board/queue rebuild. Score 10,800, chains 19, reshuffles 1, collected
  `[0,0]`, `remaining_ms` 222,399 = 214,399 + 8,000 (the live save-079 value
  220,560 is 1,839 ms of wall time later). `board`, `idle`, `tile_queue`,
  `random_queue` and `rng` all equal captured `state-079.json`.
- `captured_chain19_advances_no_ore`: same chain on state-078 plus one distant
  legal move (so no reshuffle) leaves all six ores at their exact cells with
  `idle=true`; no ore reaches the bottom row; collected stays `[0,0]`.
- `captured_states_have_no_bottom_row_ore`: 165 captured mining boards scanned,
  0 ores on the bottom row (playable rows 3-10).
- `captured_pre_reshuffle_state_is_spec_valid`: chain cells
  (10,3),(10,4),(10,5),(9,5) are all 401; the 602 ore at (10,6) sits *below*
  the cleared cells, so gravity cannot move it; six ores, all idle.

Capture chain map (full `analysis/application-*.bin`, 41 `OnChainMsg`):
#19 is `application-1321.bin` -> `10,3,10,4,10,5,9,5,`; responses
`application-6726.bin` `21 600 0` (total 10,800), `application-6727.bin`
`25 8 1`, `application-6728.bin` `11 1` (reshuffle 1). Reshuffles 2 and 3 follow
chains #26 and #34 (`application-6761.bin`, `application-6800.bin`). The
`frames/` index only holds 2 `OnChainMsg` bins (003836, 004010); the analysis
directory has all 41.

## Original ordering anchors

- `core/ServerGrid.as:530` `doMatch3` -> `updateCurrentGridArray()`; in that
  function `tile.update()` (733), the exploded-fall pass (740-749), `DumpGrid`
  (750), `if(!HasMove()) LevelReshuffle()` (751-753) and only then
  `checkFalls()` (755-759). The no-move decision happens **before** the fall
  check at the same call site.
- `core/ServerGrid.as:321-335` `LevelReshuffle()` destroys every `aTiles` entry,
  collectables included, then rebuilds the map. That matches the recovery's
  `initialize_board(..., reshuffle=true)` (`src/mining.rs:616-624`,
  `1068-1081`).
- `core/ServerTile.as:120-135`: `addFall()` only bumps `nRowsToFall` and sets
  FALL; `nY` changes only in `onFallFinished()`. So the original's 751 no-move
  test reads pre-fall positions and a reshuffle can fire while a fall is still
  animating in the original as well.

## Crafted bounded counterexample (private probe only)

Two probe tests keep the pre-existing crafted, spec-valid board: a 601 at (4,6)
above a cleared 4-chain. With a distant legal move, the ore lands on the bottom
`idle=false` and a client `OnLevelEventMsg 50` collects it exactly once. With no
legal move, the same handler reshuffles and erases the ore; the later fall
report fails ("not a goal collectable"). This is the engine's own no-move
ordering, consistent with the original site above - not a captured defect, not
evidence against the live run, and no source patch follows from it.

## Limits

Boards are saved snapshots (165 of 169 states carry a 14x14 mining board), not
every frame; the capture is a live paused run cancelled from the dev chat
(no TRY AGAIN / result close), not a closed result. Smart App Control blocked a
freshly created serde build script (os error 4551) in the probe's own target
dir at 07:2x local; the green run reused the prebuilt shared target dir and the
new test exe executed normally at 07:31 local. `frames/live-index.jsonl` is a
partial index - the analysis bins are authoritative for message counts.

## Evidence hashes (sha256)

| artifact | sha256 |
| --- | --- |
| states/state-078.json | 73c635bda2b1223e7a4b59c7bc53d12d8f18dd62150bf31893251ffd8867fc58 |
| states/state-079.json | 861bcc5cee7b45ec7e4cd4c9619ca99fcfb280651ea15bb28d631f25caf05c42 |
| analysis/application-1321.bin | c27c84929d8b19adacddec24134e5395818773d1f6389a54ed1959d6903a0f59 |
| analysis/application-6726.bin | c462ea605ef350f8cc34338a10bf5522144ef2f083dcd9276383d5b3ea56474f |
| analysis/application-6727.bin | 08ce523a8c99c688563da37b3bf6938268d6f31006818f477131e5a90c2073f6 |
| analysis/application-6728.bin | edce59468db428d281d652113c208bb9d40da9f3b68399cafd6720f7ec790c13 |
| run/server-stdout.log | a5d48b537168d29359187ab2da54f9db398a64ede0391acc7a57b71a9fa32e5e |
| core/ServerGrid.as | e1cf90298d90b97144f6e468fa5efcae87e45ba89f6023b9addd2a04a6a2c50a |
| core/ServerTile.as | 1064cc9cb80708e39860dd78a4732a1b54c3ced749b4e53f04c4c85610ae122a |
| __Packages/GameLevel.as | b1ea788d901b8db54e20ec82d4f42d512608cf10fa5822408abab9ff826f1540 |
| src/mining.rs (reviewed) | 6c59ae0ce97a998c062e5cc25658c0c4c8accd150e8db0ac7550c756c1ed38ed |

Machine-readable bundle: `lab/reports/mining-pending-fall-reshuffle-20261009T140740Z/proof-manifest.json`.
