# Mining Native 151307 Final Review

Independent read-only final audit of `lab/runs/client-baseline-20261009T151307Z`.
Writes are this note and `lab\reports\mining-native-151307-final-review-20261009T161959Z/` only; no UI, process,
release, save, content or source edit.

## Verdict

Capture, state conservation, driver accounting and media finalization all
re-derive. The run ends **Lost by the wall-clock cap** (not a win, not a forced
quit), the engine counted **85 chains against 84 guarded driver chains**, and
every other gameplay section is unchanged; character position/rotation changed.

## 1. Frames / analyzer

`AUDIT_OK=true` (my own re-parameterized audit, `audit.log`):

- 22,094 frames = 22,043 gateway + 51 login; 0 length/hash mismatches, 0
  unlisted/missing.
- CRC length 3; 22,041 non-handshake gateway frames CRC3+opcode verified; 0
  decode errors; reassembly complete both directions (9,923 + 208 duplicates
  client→server, 463 server→client); 10,595 applications; created-self 154,395 B
  `09d96f77…`.
- `analysis/gateway-decode-proof.json` fields all equal mine (8/8).

## 2. Final state (no win)

`character-after.json` (sha `7a0fc38a28fcc94b191fcc1a0163cb77a9e801b8cbb34b9ce8cb013ff384a2da`,
identical to the branch state file) carries mining `phase=Lost`,
`wall_ms=1,800,000` (the engine cap), `remaining_ms=597,449`, `end_sent=true`,
`chains=85`, `score=54,200`, `collected=[2,0]`, `reshuffles=5`. The last paused
snapshot `character-mining-before-quit-chain84.json` (15:54:04Z) has the same
chains/score/collected at `wall_ms=875,511` and `phase=Playing`; no win or
`OnGameWon` path appears.

## 3. Driver chains: 84 guarded vs 85 engine

`batch1` failed at `Offered` before any input; `batch2..13` each executed
exactly 7 guarded chains (12 × 7 = **84**), with contiguous
`chains_before/chains_after` 0→84 and no gaps. The engine counted **85**: the
extra `84→85` transition is save line 746 (`wall 600,860 → 763,334`,
`score 53,800 → 54,200`) and has no driver receipt. Its origin is not proven;
the gap is recorded, not explained.

## 4. Conservation

Before `e8c40a2e…` → after `7a0fc38a…`: exactly five changed paths —
`/character/position`, `/character/rotation`, `/gameplay/harvesting` (section
added), `/gameplay/mining/active`, `/gameplay/mining/next_instance_id`.
`inventory`, `profiles` (active 14, unlocked, experience), `identity_fingerprint`,
`combat`, `cooking`, `forging`, `smelting` are unchanged, and so is `opening`:
its `stages` carry 17 entries with all **17 `rewarded=true`**
(910001…910017) and are byte-identical before/after, with equal `discoveries`
and content hash — the semantic local-stage coverage is unchanged even though
no literal `local*` field name exists, and that literal absence does not negate
the coverage. `quests` is unchanged as well, with the qualification that it is
a historical subset (2 entries: 1617 accepted-not-completed, 900001 not
accepted), not the full quest catalogue. identity-before == identity-after
(`76c814e8…`).

## 5. Media and shutdown (bounded)

- `native-mining-harvest.mkv` 47,916,171 B with a valid EBML header, and
  `native-mining-harvest.audio.raw` 784,055,568 B = **2,041.811 s** at 48 kHz
  stereo f32; ffmpeg log starts 15:39:15Z and the pair finalizes 16:13:16Z,
  consistent with the recording length. Validated by metadata/header/size, not
  decoded frame-by-frame.
- No `.stop` marker exists in the run and recorder PID 25343 is not recorded
  there; recorder completion rests on the finalized media plus the manifest.
  Manifest `elapsed_seconds=3600.016` (1 h self-end); `client_exit_code=None`;
  client/login/server all `stopped=true`; login ends `session_ended /
  client_disconnect`.
- No world screenshot exists in the run; registered screenshots are the
  client-* set, and the last mining screenshots are `mining-after84.png`
  (15:49:58Z) and `mining-quit.png` (15:54:07Z, the paused 54,200 board). No
  evidence of a world return is claimed.

## Limits

Media validation is metadata-level; the unattributed 85th chain is unproven;
all raw bytes and full inventories stay in the lab (no raw capture or code is
copied into project notes).
