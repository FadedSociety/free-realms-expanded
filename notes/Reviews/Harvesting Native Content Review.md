# Harvesting native content review

Status: independent read-only review of `lab/references/opening-harvesting-native-20261009T132512Z`
against the frozen source package `lab/references/opening-mining-native-20261009T121330Z`, run
2026-10-09 13:38-14:0x UTC. Evidence label: **ORIGINAL package bytes and catalogs, independently
re-derived; RECONSTRUCTED content judged as reconstructed.** No native acceptance is claimed, and
the native gate is currently **blocked** (finding 1).

Owned paths written by this review:

| Path | Role |
| --- | --- |
| `notes/Reviews/Harvesting Native Content Review.md` | this review |
| `lab/reports/harvesting-content-review-20261009T133854Z/` | probes, logs, JSON proofs, hash manifest |

No production, content, save, package, process or UI file was edited. The package files were read
only; every probe writes inside the review's own lab directory.

## Verdict

The catalogue, inventory-transaction and reward-lifecycle claims in
`notes/Research/Harvesting Native Content.md` hold up under independent re-derivation: the four
bundle definitions resolve exactly once, their wire bytes are reproduced byte for byte from the
declared oracles, the inventory change is an append that preserves every prior record, and the real
gateway grants each bundle once and only once with coins untouched and a replay inert.

Phrase-level corrections are needed in the builder note (findings 2-4) and one **blocking native
gap** is recorded (finding 1): the package's tile information does not cover the regular selectable
tiles, so the reconstructed content cannot yet render them in the original client movie.

## Independently re-derived

| Claim | Method | Result |
| --- | --- | --- |
| Package file accounting | sha256 of every file vs the frozen base | **42** candidate files, 38 base: 37 identical, 1 modified (`content/inventory.json`), 4 added (`STAMP.txt`, `content/harvesting.json`, `harvesting-content-proof.json`, `harvesting-receipt.json`) |
| Inventory 1701 -> 1703 | raw JSON span comparison (`inventory-proof.py`) | prior 1701 rows byte-identical in order; appended rows are exactly `[2885, 2890]`; no duplicate id introduced (base already carries 43 identical duplicates, candidate carries the same 43) |
| Untouched document members | raw span compare | `schema_version`, `player_guid`, `default_state`, `profiles`, `spans`, `initial_state` byte-identical; `provenance` gained one member (`harvesting_supplement`) |
| 2887 / 2877 are original rows | oracle `ClientItemDefinitions.json` sha256 `a001f81a...` -> `definition_bytes` | stored hex and sha256 match the recomputed wire exactly; unchanged between base and candidate; oracle rank 0 / resell 1 |
| 2885 / 2890 wire bytes | Legacy `40a3cc79...` and Sanctuary `ffe9cd2d...` catalogs -> `legacy_definition` -> `definition_bytes` | both catalogs agree byte for byte; produced row equals the stored row field for field; wire sha256 `dc9dafec...` / `44a6c7d5...` |
| Four outputs once | candidate definition scan | `2885`, `2887`, `2890`, `2877` each appear exactly once |
| `harvesting.json` | bytes + JSON | 3734 bytes, sha256 `280f5cd7...`; game 111, profile 45, name 100834, description 94150, icon 908, `FarmingAssets.txt`, `game_farming.gfx`, outputs `[2885, 2887, 2890, 2877]`, goals 601..604 at quota 15 / per_reward 5, `reconstructed_rules: true`, zero "stew" references |
| Real loader binds the package | `Gameplay::load` / `Type8Engine::load` on the candidate | binds game 111 / Chef 45 and the four output ids; the source package has no harvesting content and lacks 2885/2890 |
| Reward lifecycle | real `Gateway` win on a scratch copy | each bundle granted exactly once, coins unchanged, exactly +4 item rows, forged client win inert, replay inert, restart with the post-win state keeps exactly one of each |
| Conflicting duplicates | real loader, scratch copies | a byte-identical duplicate row still loads; a structurally valid but different duplicate row fails with `conflicting inventory definition` |

The builder's own preflight (`lab/reports/harvesting-native-content-20261009T132512Z/probe`, 4
tests) was **not** treated as evidence: only my own probe project
`lab/reports/harvesting-content-review-20261009T133854Z/probe` (3 tests, `review_probe.rs`) is
cited, run against the same public API.

## Findings

1. **Blocking - native readiness: regular selectable tiles have no tile information (root-verified,
   independently confirmed).** `content/harvesting.json` lists `selectable: [401, 402, 403, 404]`
   but `spec.tiles` only covers the collectables 601..604; no `OnTileInfo`/tile-info block exists
   anywhere in the spec. In the original harvesting movie
   (`lab/decompiled/minigames/harvesting/scripts/scripts/__Packages/core/TileGenerator.as`),
   `Initialize()` populates `aNames` only for engine tile constants, and `generate_selectable`
   dereferences `this.aNames[_nValue]` -> `getMovieClipByObject(...)`; values 401..404 get an
   `aNames` entry only through `SetTileInfo` (the client message path, obfuscated as `n(value,
   type)` in `GameClient.as`). With the current package those lookups are undefined, so regular
   selectable produce will not render correctly and the native gate is blocked. Fixing the module
   and emitting a new package serial is the lead's separate follow-up; this note changes nothing.
2. **File accounting.** The note says "41 files ... 37 identical / 1 modified / 3 added". The
   package actually holds **42** files with **4** added; `STAMP.txt` is excluded from the note's
   count without saying so. The builder's own `file-proof.json` records `added: 4`, so only the note
   summary is off. Recommended wording: "41 content files plus the `STAMP.txt` marker (42 files, 4
   added)". No package change needed.
3. **Serializer invocation detail.** The note implies the stock community chain ran unchanged. The
   stock `prepare-opening-base.legacy_definition` -> `definition_bytes` pair raises
   `KeyError: 'TintId'` on these catalog rows because `Icon` carries only `Id`; the invoked chain
   normalizes `IconData` to `{Id, TintId: default 0}` (builder script lines 96-99). That
   normalization is consistent with precedent - the already-accepted cooking row 35 reproduces
   byte-exactly under it - but the note should name it rather than imply an unmodified helper ran.
4. **Provenance member not mentioned.** The inventory diff is not only two definition rows plus a
   root hash change: `provenance.harvesting_supplement` is appended. It follows the established
   `cooking_supplement`/`forging_supplement` shape and is good disclosure; the note's "untouched"
   list should either include `provenance` as intentionally extended or exclude it explicitly.
5. **Rank/resell wording.** The disclosed difference is real: 2885/2890 (community) come out with
   `MinProfileRank = 1` and no `ResellValue` field at all, which the mapping defaults to 0, while the
   supplied 2887/2877 rows carry rank 0 / resell 1. "stores `ResellValue=0`" is therefore a default,
   not a stored catalog value; "derives 0" is the accurate phrasing.

## What these probes do not prove

- No native client run was made and none is claimed. The gateway/loader probes are headless.
- The tile-information gap (finding 1) means the movie-level rendering of regular selectable tiles
  is **unverified and currently unreachable** from this package; quotas, rewards and transactions
  are unaffected by that gap.
- No world entry or trigger for region 111 is recovered; the catalogue/launch binding is not a
  trigger.
- `2885` and `2890` remain community-catalog reconstructions; only `2887`/`2877` are supplied
  original rows.
- The 39/16 reward payload and 26/12 launcher remain locally authored; no original capture exists.

## Frozen source hashes at probe time

The gameplay guard/source and mining-parity work was in progress while this review ran; these are
the hashes the probe actually compiled against (`hash-manifest.txt` records the full list):

| File | sha256 (prefix) |
| --- | --- |
| `src/gameplay.rs` | `996d33865966fcf8` |
| `src/inventory.rs` | `dccbf8006bd4d819` |
| `src/mining.rs` | `6c59ae0ce97a998c` |
| `src/harvesting.rs` | `272711da0e6b72fc` |
| `src/gateway.rs` | `60110c143402fc1c` |
| `src/lib.rs` | `bb1d8755a8b6a606` |

`mining.rs` was still changing when the probe ran, so the mining-parity result is not frozen to
this hash beyond the recorded run.

## Commands and results

All commands ran from `%USERPROFILE%\Documents\Projects\CartoonMMO` with the private toolchain and
the pinned lab Python (`lab/tools/wiki-env/Scripts/python.exe`).

| Command | Result |
| --- | --- |
| `python lab/reports/harvesting-content-review-20261009T133854Z/inventory-proof.py <base> <candidate> <out>` | 1701 -> 1703, prior rows byte-identical, appended `[2885, 2890]`, no new duplicate ids; `provenance` flagged as the only changed member (`inventory-proof.log/.json`) |
| `python .../wire-proof.py ...` | oracle, Legacy and Sanctuary sources hash-verified; 2887/2877 hex+sha match; 2885/2890 both catalogs agree and match stored rows field for field; cooking control reproduces; `ok: true` (`wire-proof.log/.json`) |
| `cargo test --test review_probe` in `.../probe` with `CARGO_TARGET_DIR=.../probe-target` | **3 passed, 0 failed**: bindings + source control; win once-only, replay inert, coins unchanged, restart keeps one; identical duplicate tolerated, conflicting refused (`probe-run.log`) |
| `python .../hash-manifest.py .../hash-manifest.txt` | 42 candidate / 38 base files: 37 identical, 1 modified, 4 added, 0 removed |

Two probe defects were found and fixed during the run and are retained as evidence: the first
conflict mutation changed the wire id and hit the loader's content check (`invalid inventory
definition content`) instead of the conflict check, so the mutation was moved to the trailing
integer; and the first review attempt assumed the 41-file count before enumerating the directory.

Skills applied: **RTK** (normal commands, condensed evidence; no `rtk proxy` needed), **ponytail**
(three small probes reusing the real loaders and the project's own serializer instead of new
machinery), **caveman** (terse reporting; this persisted review stays complete), **asset-pipeline**
(applied as the asset/movie readiness check that produced finding 1: the movie's own tile-name
table is the authority for what the client can render). `game-automation` was not applied - only
the lead controls the native window, and this item explicitly makes no native claim.

Cross-links: [[Harvesting Native Content]], [[../Design/MMO Worker Contract]], [[../MMO Work
Queue]], [[Housing Native Content Review]].
