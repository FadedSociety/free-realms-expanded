# Mining Short Idle Acknowledgement Review

Independent read-only review of the Mining short idle acknowledgement (`Grid.UpdateIdle` →
`GameLevel.OnIdleMsg` → `ServerGrid.NowIdle`) against the current Rust `mining.rs` gate. Writes are
this note and `lab/reports/mining-short-idle-review-20261009T1742Z/` only; the mining writer owns
`src/mining.rs`, the new idle tests and its note — no edits by this reviewer. Skills: RTK (normal
commands), Ponytail (reads only, no new tooling), Caveman (terse), reverse-engineering (decompiled
movie anchors).

**Status: original semantics verified below; the Rust repair source pin has not arrived yet, so the
post-pin probes in section 3 are still pending.**

## 1. Original semantics (verified from the decompiled movie; hashes in the report)

| Anchor | Evidence |
| --- | --- |
| `core/Grid.as:518-521` | `nowIdle` appends `x,y,` per idle tile into `sNowIdle` |
| `core/Grid.as:1356-1363` | `UpdateIdle` sends the accumulated string **only when nonempty**, then clears it |
| `GameClientNetwork.as:26-29` | `SendIdle(list)` → `AddMessage(true,"OnIdleMsg",list)` (verbatim) |
| `GameLevel.as:543-552` | `OnIdleMsg`: game-done guard, `split(",")`, **pop exactly one trailing empty**, `oServerGrid.NowIdle(list)` — **no minimum length** |
| `core/ServerGrid.as:78-102` | `NowIdle(_aIdleList)` (the lead's "SetIdle"; actual name `NowIdle`): loops `list.length / 2` pairs, flips tiles in `IDLE_PENDING`/`APPEAR` to `IDLE`; **no minimum** — 1 tile (2 values) and 2 tiles (4 values) are accepted; an empty list is a no-op |
| `core/ServerGrid.as:103-110` + `GameData.as:47` | chain contrast: `ValidateAndRemoveChain` rejects `list.length/2 < GameData.nMINIMUM_CHAIN_LENGTH` with `nMINIMUM_CHAIN_LENGTH = 3` |

**Conclusion: the original accepts ANY nonempty coordinate list for the idle acknowledgement; the
3-tile minimum belongs to `OnChainMsg`/`ValidateAndRemoveChain` only.** The idle path also changes
only tile idle state (no score, quota, chain or board mutation).

## 2. Current Rust gate and observed refusals

- `prototype/osfr-tools/src/mining.rs` `4ABE6B07…` is the source compiled into the frozen lead
  binary `59032cfa…` (per `lab/reports/lead-minigame-gate-20261009T173220Z/release.json`).
- The shared `coordinates()` (`mining.rs:2134-2158`) requires `values.len() >= 6` and is used by
  both `["OnChainMsg", …]` (line 690-692) and `["OnIdleMsg", …]` (line 776-787) — so a 1- or
  2-tile idle acknowledgement is refused as `malformed mining coordinate list`.
- `powerup_points()` (`mining.rs:2162-2181`) already uses the `>= 2` single-tile minimum, so the
  minimal repair shape is a per-caller minimum rather than a new wire format.
- Real-run evidence (`lab/reports/mining-clickable-round-20261009T1728Z`): `run2-powerup-25` and
  `run2-powerup-50` each sent `OnIdleMsg 3,3,4,3,` (2 tiles, 4 values) and received
  `{"error": "malformed mining coordinate list"}`; 37 other `OnIdleMsg` cases with ≥3 tiles were
  accepted. No 1-tile case exists in that transcript set — the 1-tile refusal follows from the same
  `len < 6` gate and must be covered explicitly after the pin.

## 3. Post-pin verification contract (pending the repair pin)

1. 1-tile (2 values) and 2-tile (4 values) idle ACKs are accepted and change **only** `run.idle`
   flags — no score/chain/quota/board/byte changes.
2. 1-tile and 2-tile `OnChainMsg` lists are still rejected (chain minimum 6 values, unchanged
   `validate_chain`).
3. Malformed atomicity: odd value counts, non-numeric, coordinate > 4096, interior empty field,
   oversize list → refused with no partial idle mutation (full pre/post run-state compare).
4. No padded ACK: the accepted wire string is not rewritten, no coordinates appended, request
   format unchanged (only the original one-trailing-comma pop contract).
5. Out-of-board idle coordinates refused with no partial mutation.
6. Regressions: prior ≥3-tile idle cases and frozen behaviours unchanged; focused mining suites
   green.

## 4. Artifacts

`lab/reports/mining-short-idle-review-20261009T1742Z/`: `original-excerpts.txt` (anchors + source
hashes), `current-source-shape.txt` (mining.rs excerpts + hash), `run-evidence.json`
(refusals/accepted counts/frozen linkage), `receipt.json`, `proof-manifest.json`,
`final-hashes.txt`.

## Pin checkpoint (repair accepted)

Pinned `src/mining.rs` `8B32D7C5…` and `tests/mining_idle.rs` `8C2A45F4…` match the lead receipt;
the receipt's untouched files also match current (`src/gameplay.rs`, `src/harvesting.rs`,
`src/lib.rs`, `Cargo.toml`). Repair shape verified: shared `coordinate_list(list, minimum_values)`
(`mining.rs:2140-2159`), `coordinates` → 6 (chain), `idle_points` → 2, `powerup_points` → 2;
`["OnIdleMsg", …]` uses `idle_points` (line 777) and the chain handler is unchanged (line 691).
Lead RED/GREEN receipts read: `red.log` 3 failed / 3 passed (pre-fix refusals), `green.log` 6/0,
focused suite summary on file.

Independent probe crate `pin/` (path dependency on `osfr-tools`, private target dir,
`cargo run --offline`): **18 checks, 0 failures**:

- 1-tile and 2-tile idle ACKs accepted; full-state equality clones prove **only the idle bit(s)
  changed**; `replies` is empty (no echoed/padded ACK); repeating the same ACK is idempotent.
- **Valid-first-then-outside `OnIdleMsg 4,4,0,0,` → `Err` with the full state unchanged**
  (rollback after the would-be `4,4` mutation), complementing the builder's outside-first test.
- Bounds: `0,0`, `11,3`, `4,11` refused unchanged; `3,3` and `10,10` accepted with only that bit.
- Short chains still refused (1 and 2 tiles, state unchanged); the 3-tile chain still accepted.
- Malformed matrix refused with full state unchanged: odd count, interior empty field,
  coordinate > 4096, non-numeric, bare comma, `4,4,,`, negative, longer odd list.
- Phases: `Won`/`Lost` accept a well-formed ACK but change nothing (no replies); paused `Playing`
  still applies; paused malformed refused unchanged; no-active-run engine returns the default
  outcome with state unchanged for both valid and malformed input.

**Findings: doc-only nit** — `mining.rs:2162` says `GameLevel.ValidateAndRemoveChain`; the actual
function is `ServerGrid.ValidateAndRemoveChain` (`ServerGrid.as:103-133`). The quoted semantics are
correct. No behavioural findings; **repair accepted**.
