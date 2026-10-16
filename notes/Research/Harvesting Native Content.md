# Harvesting Native Content

Private native harvesting content package for the opt-in Harvesting111/Chef45
gateway candidate. Nothing here changes shared content, saves, releases, the
launcher, processes or UI; native acceptance remains the lead's gate. Frozen
sources at freeze time: `harvesting.rs` `272711da0e6b72fc`, `mining.rs`
`e6e421d8e7cd0e92`, `gameplay.rs` `9e8b56ed5f3a2f78`, `lib.rs`
`bb1d8755a8b6a606`.

| | |
| --- | --- |
| Package | `../lab/references/opening-harvesting-native-20261009T132512Z/` |
| Source | whole-package copy of `../lab/references/opening-mining-native-20261009T121330Z/` |
| Evidence bundle | `../lab/reports/harvesting-native-content-20261009T132512Z/` |
| File proof | 37 identical / 1 modified (`content/inventory.json`) / 3 added; all 38 frozen source files re-hashed unchanged |
| Inventory | `1f8733ea386d2181c388339f676e63900059efdd7a1f68a2d488f0df07b07dc5` (1701) -> `0b95b0b1c032a2e0f864f8227bfe693fe58181863eaf182d35b6b9dbc4d9877a` (1703) |
| `content/harvesting.json` | `280f5cd7fcca711aaf7e8118ef04195b3bb1241413498325288aca10dff94d20`, 3734 bytes, written by the verified writer `harvesting::write_content(dir, 111)` |

## Only two bundle definitions were missing

The immutable base inventory already defines 2887 and 2877 byte-exactly; only
2885 and 2890 were absent, so only those two rows were appended (1701 -> 1703).
The earlier "four missing" instruction was a summary error and is superseded.
Appending community rows for 2887/2877 would not "shadow" anything: `src/inventory.rs:170`
fails a whole `Inventory::load` with `conflicting inventory definition` when a
repeated id differs, and the probe reproduces exactly that counterexample.

| id | item (recovered catalog) | source of stored bytes | wire sha256 | record sha256 |
| --- | --- | --- | --- | --- |
| 2885 | Bundle of Stickfruit, icon 715, name 23295 | community Legacy catalog `40a3cc79…` (`legacy_definition` + `content_definition`) | `dc9dafeca11f66e8…` | `43e7229fcbdfdad2…` |
| 2887 | Bundle of Stainberry, icon 713, name 23297 | supplied definitions `a001f81a…` (already present, re-verified identical) | `6d08cb251bb51469…` | `8ea6e8ecee6ab0f8…` |
| 2890 | Bundle of Tumnip, icon 717, name 23300 | community Legacy catalog | `44a6c7d59dd3e541…` | `338a6164db7547a0…` |
| 2877 | Bundle of Spiralmint, icon 712, name 23287 | supplied definitions (already present, re-verified identical) | `ff20e50f75300fba…` | `dd5e23b691ae0085…` |

Full `serialized_hex`, typed metadata and hashes: `four-output-availability.json`
and `apply-proof.json` in the evidence bundle. The serializer is the existing
community/verified chain `prepare-gameplay-inventory.py definition_bytes` driven
by `prepare-opening-base.py legacy_definition/content_definition`, exactly as
the cooking and forging supplements were built. Sanctuary's
`ClientItemDefinitions.json` independently produces identical wire bytes for
2885/2890.

Original versus reconstructed is explicit: 2887/2877 are supplied original
records; 2885/2890 are community-catalog reconstructions. The community mapping
stores `MinProfileRank=1`/`ResellValue=0`, while the supplied sibling rows carry
`MinProfileRank=0`/`ResellValue=1`; that two-int difference is reported, not
silently harmonised.

## harvesting.json is unchanged reconstruction

Game 111, Chef45, name 100834, description 94150, icon 908, preload
`FarmingAssets.txt`, movie `game_farming.gfx`, outputs
`[2885, 2887, 2890, 2877]`, goals 601..604 at quota 15 / per_reward 5, and
`reconstructed_rules: true`. This content edit does not change the emitted
board, quotas, cadence, produce set (no six-produce variant) or any stew
coupling; the quest gather path is untouched.

## Evidence

| File | Proves |
| --- | --- |
| `check-before.log` / `check-before.json` | 2885/2890 missing, 2887/2877 identical to oracle, before any edit |
| `apply.log` / `apply-proof.json` | appended ids, prior-bytes-as-prefix proof, untouched `default_state`/`profiles`/`spans`/`initial_state`, oracle hex/sha/typed metadata |
| `idempotence.log` / `idempotence-proof.json` | rerunning the append verifies all four ids and leaves the file byte-identical |
| `unknown-id-refusal.log` | an id with no definition is refused with an explicit error and the target is unchanged |
| `preflight.log` | private probe 4/4: real engine+gameplay load of the package; old save without `harvesting` stays byte-identical and `inventory`/`quests`/`combat`/`profiles` equal the source package; a gateway win grants each real bundle once (forged win inert, coins unchanged, exactly +4 rows, replay grants nothing); a conflicting duplicate row is refused |
| `four-output-availability.json` | post-edit re-verification of the file actually written: 1703 definitions, prior-rows prefix preserved, all four ids resolve exactly once with oracle-matching hex/hash and are bound in `harvesting.json` |
| `file-proof.json` | per-file identical/modified/added classification and re-hash of the frozen source |

Commands: `python append-harvesting-definitions.py --inventory <package>/content/inventory.json --apply` (three times: check, apply, idempotent rerun), `cargo run --bin write-harvesting-content -- <package>/content`, `cargo test --test package_preflight`, `python verify-four-outputs.py`, `python verify-package-files.py`.

## Skills applied

RTK (normal commands, condensed evidence); Ponytail (reused the existing
serializer/generators and the real gateway tests instead of new machinery);
Caveman (terse reporting); reverse-engineering (reused released serializer
evidence and pinned catalogs; no new format reversed); asset-pipeline (no art
generated — icon and name references come only from the pinned catalogs).

## Remaining gaps

- The package is private and unregistered; no shared content, save, release,
  launcher, process or UI change was made.
- 2885/2890 are community-catalog reconstructions; only 2887/2877 are supplied
  original bytes.
- The 39/16 reward-bundle payload and 26/12 launcher remain locally authored;
  no original capture exists.
- No world entry/trigger for Harvesting111 is recovered; the activity catalogue
  is not a trigger.
- The reconstructed four-bundle output set, quota and cadence stay as reviewed;
  the locale-vs-wiki six-produce question remains open and was not acted on.
