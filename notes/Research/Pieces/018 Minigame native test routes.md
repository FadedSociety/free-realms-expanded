# 018 Minigame native test routes

Status, October 9, 2026: developer entry points implemented as a route
catalogue and a `realm game` resolver, gated off by default, verified by 13
Rust tests (11 ordinary, 2 private-content) and 15 Python tests plus a
read-only checker that re-derives every route's evidence. **No native client
run was made by this worker.** Nothing here is a playability claim: the three
crafting routes reuse the already-reviewed `realm goto` relocation, and the
other six routes will not move the character at all.

## What this piece delivers

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_018.rs` | The nine-route catalogue, the exact/alias/unique-prefix resolver and the gated `handle` (not yet registered in `lib.rs`) |
| `prototype/osfr-tools/tests/piece_018.rs` | 13 owned tests: gate default-off, table coverage, catalogue/file binding, resolver semantics, refusal paths, composition with the reviewed `dev_teleport` standoff, and two ignored private-content checks |
| `scripts/pieces/piece_018.py` | Read-only checker: parses the Rust table, re-binds every route to the item-015 catalogue, the module constants, the bound content actors and the production gate literals, then writes the evidence bundle |
| `tests/pieces/test_piece_018.py` | 15 stdlib tests: parser failures with line numbers, unknown/wrong-type game ids, actor mismatches, gate tampering, the recorded conflict finding, real-source audit and fixed-stamp determinism |
| `../lab/reports/piece-018-20261009T025834Z/` | RED/GREEN logs, `manifest.json`, `findings.json`, `provenance.json` (gate-source hashes), `run.log`, `final-hashes.txt` |

## Route table

`status` is this repository's acceptance state, never a historical claim.

| `realm game` route | Status | Cited recovery | Entry point |
| --- | --- | --- | --- |
| `cooking` (`cook`) | native-accepted | game 46, TYPE 9 mimic movie | world actor `Cooking Table`, profile 45 Chef, recipe 2001 |
| `smelting` (`smelt`) | native-accepted | game 77, TYPE 9 mimic movie | world actor `Smelt 5 Copper Bars`, profile 14 Miner, recipe 2002 |
| `forging` (`forge`) | native-accepted | game 139 (see conflict below) | world actor `Anvil`, profile 16 Blacksmith, recipe 2003 |
| `mining` (`mine`) | code/test candidate | games 18 and 91, TYPE 8 MatchThree | module driver `--test mining` |
| `harvesting` (`harvest`) | code/test candidate | games 58 and 111, TYPE 8 MatchThree | module driver `--test harvesting` |
| `fishing` (`fish`) | code/test candidate | games 595 and 596, TYPE 21 | module driver `--test fishing` |
| `kart_racing` (`kart`, `racing`) | code/test candidate | games 28, 37, 38, TYPE 10 | module driver `--test kart_racing` |
| `soccer` (`football`) | code/test candidate | games 42 and 61, TYPE 12 | module driver `--test soccer` |
| `derby` (`demolition`) | wire study only | game 39, TYPE 11 | no launch route; reports the missing contract |

## Original facts, inferred meaning, reconstructed policy

Recovered original facts cited by the routes, unchanged from their own notes:
the nine catalogue rows and their declared types (item 015); the movie ids
`const GAME` in `cooking.rs` (46), `smelting.rs` (77) and `forging.rs` (139);
the recovered packet families (`39/x` TYPE 8/9, `138` fishing, `76` soccer,
the vehicle-race and derby subtype streams, `127` housing); the recovered
chef/miner/blacksmith profiles (45/14/16) and recipe ids (2001/2002/2003) that
the bound content binds to the three crafting actors.

Inferred meaning, kept as reconstruction: the family labels
(`TYPE 9 mimic movie (Chef)` and so on), the "code/test candidate" reading of
the unregistered modules, and the claim that an actor interaction is the
historical entry point for cooking/smelting/forging. The last one rests on the
accepted production integration and the recovered opening content, not on a
recovered native tutorial script.

Reconstructed policy, ours: the route names, aliases, the `realm game`
contract, the reply wording, the requirement that the entry actor is loaded in
zone 0, and the reuse of the reviewed 4 m `dev_teleport` standoff for the
arrival. The module never computes its own geometry and never sends a packet.

## The one recorded conflict

`forging.rs` implements the native-accepted Spunky Scrapper anvil movie with
`const GAME: i32 = 139`, but catalogue row 139 is
`Forging: Spunky Scrapper Hammer`, declared `TYPE 8` in slot
`forging-boards`, while the accepted movie family is `TYPE 9` (mimic anvil).
The Rust table cites the module constant as the id authority and the checker
records the mismatch as a finding instead of silently rewriting either side:

```
forging is an accepted native movie but its cited game 139 is catalogue TYPE 8
(forging-boards); the module constant is the authority for the id and the
conflict needs lead reconciliation
```

This is the only finding in the bundle; a missing or extra finding fails the
Python tests.

## Gate default-off

`handle` returns `None` before it parses anything while `enabled` is false, so
`realm ` commands keep their existing handling byte for byte. The checker
re-reads five production sources and fails if any reviewed literal disappears:

| Source | Required literal |
| --- | --- |
| `src/gameplay.rs` | `dev_tools: false,` and `self.dev_tools && self.developer_teleport(` |
| `src/gateway.rs` | `dev_tools: false,` |
| `src/bin/osfr-gateway.rs` | `let mut dev_tools = false;` and `if key == "--dev-tools" {` |
| `scripts/run-local-baseline.py` | `"--dev-tools"` and `if run_options.dev_tools:` |
| `src/restoration/piece_018.rs` | `if !enabled {` and `return None;` |

The SHA-256 of each checked source at this revision is in
`../lab/reports/piece-018-20261009T025834Z/provenance.json`.

## Command contract

`realm game` and `realm game list` print the table; `realm game <slug or
alias>` resolves exact slug, then exact alias, then a unique prefix, and
answers with candidates when a prefix is ambiguous (`s`, `f`) or unknown.
For a crafting route the reply names the actor, its GUID from the loaded
content, the required profile and recipe, and asks the caller to relocate;
the caller applies the reviewed command
`dev_teleport::handle("goto <actor>", actors, character)`, which returns the
`Relocate` command. For the module-driver routes the reply prints the module,
the driver command and the open native gate; for `derby` it prints the missing
launch contract. Missing or duplicated entry actors are refused with the
character unchanged.

Lead integration (this worker may not edit `gameplay.rs`): inside the
existing `self.dev_tools` branch, convert the loaded `OpeningActor` rows to
`piece_018::ActorView { name, guid, zone_id }`, call
`piece_018::handle(true, command, &views)`, and route a
`Command::Route { actor, .. }` through the same relocation path
`developer_teleport` already uses. Register the module as
`pub mod piece_018;` under `src/restoration/` and drop the `#[path]` include
in the test.

## Verification

```powershell
# RED: naive baseline (empty table, gate ignored) against the real expectations
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_018
# 4 passed, 7 failed, 2 ignored  -> red-01-naive-baseline.log

# GREEN: the implemented module
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_018
# 11 passed, 0 failed, 2 ignored    -> green-01-ordinary.log
& scripts/cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test piece_018 '--' '--ignored' '--test-threads=1'
# 2 passed, 0 failed                -> green-02-private-content.log

# Checker: RED without the script, then the real audit and bundle
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_018.py" -v
# 15 tests, OK                      -> green-04-python-tests.log
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_018.py --stamp 20261009T025834Z --reports-dir ..\lab\reports\piece-018-20261009T025834Z
# routes=9 errors=0 findings=1      -> green-03-checker-cli.log
```

Bystanders recorded for the lead: the ordinary library suite is
**24 passed, 0 failed** (`green-06-lib-suite.log`); the repository-wide
`cargo test` currently fails to compile other workers' in-flight targets
(`piece_008.rs` and `piece_016.rs` cite modules that are not written yet,
`green-05-full-suite.log`), and the Python `tests/pieces` discovery is
**152 passed, OK** once those peers stop writing (`green-08-python-pieces-suite.log`).

Input provenance: `notes/Research/original-activity-coverage.json` sha256
`58ec6a2a45f905cdec10b259b23dd8c77010fd751d85486cc965993265407dda` (the value
item 015 pinned); bound content `opening-anvil-placement-v1/content/opening.json`
sha256 `4e6ac15cd19b5785a04ac7eb944bf0235f4a0592f4c42df755f89c47896b0ad1`
(the value the opening ledger pins). Artifact hashes at this revision:
module `a1b1aa299ca6cf00b5d4dcd8fde47da7152734f225f14610e0018ec4c2e4717f`,
test `138a635566a6bdc25645c2f6a8ba897e07555a7e27fb81c1a5b0e0ae2b20b91a`,
checker `cd5bf53efc6712b6f216c6e3b6039ba4a7654ec666e09cc8132b38507a1b3a93`,
checker tests `36b85119c0cf64b442c60c87c7951a2ea1e6a107cdf01cdf1dca53e9688bc525`.

## Remaining gates

- Native: a human/lead run with `--dev-tools` must show `realm game list`,
  `realm game cooking` reaching the Cooking Table and starting the accepted
  movie, and `realm game derby` refusing. This worker did not touch the
  client, the launcher or the release build.
- Integration: register the module, convert actors to `ActorView`, and apply
  the returned relocation. Until then the routes are reachable only through
  the owned tests.
- Evidence: the module-driver routes stay candidates until each module is
  registered and driven by the gateway; `derby` has no launch contract at all
  (item 007's note records the recovered opaque frame study).
- The forging id/type conflict needs a lead decision; the checker keeps
  failing loudly if the finding disappears.

## Skills applied

`game-recon` for the native-versus-Flash split that drives the status column,
`reverse-engineering` for reading the recovered module constants, catalogue
rows and content bindings as the specification rather than guessing entry
points, `game-research-websearch` practice (community corpus first) only to
confirm the lane, plus RTK (normal commands first), Ponytail (the route table
reuses the reviewed `dev_teleport` standoff instead of a second sampler and
adds no dependency) and Caveman (concise reporting, this note kept complete).
`asset-pipeline` and `game-automation` were not needed: no asset conversion
and the shared native window stays under the lead's control.
