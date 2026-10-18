# Playtest Bundle

W122 packages a private, local playtest input set outside the Git repository.
The builder defaults to `../lab/bundles/playtest-<UTC timestamp>/`; it reads
repository content from the committed `HEAD`, overlays the allowlisted content
from `lab/reports/round70-native-20261016T061000Z/content`, and records the
source commit in the bundle README. It does not modify the active play package.

## Contents and provenance

- `content/`: tracked `prototype/osfr-tools/data/content` from `HEAD`, followed
  by the explicit runtime allowlist from the active round70 package. The active
  `inventory.json` saved `initial_state` is removed so the runtime uses its
  packaged default. `character_json_sha256` is removed from package provenance;
  other content provenance is retained. Prior saves, proofs, captures and
  package reports are not copied. `combat-template.bin` and
  `encounter-template.bin` accompany their JSON definitions; each config pins
  its template hash, and the Rust loaders require those packet templates.
  Activity content is tracked, not overlaid: `kart_entry.json`, `kart_lobby.json`,
  `kart_gates.json`, `mining.json`, `harvesting.json`, `encounters.json`,
  `encounter-template.bin`, `fishing_spots.json`, `mining_nodes.json` and
  `harvest_nodes.json` come from the committed content tree (T026). The builder
  and checker validate these files before a bundle is used; the exact bytes are
  pinned by `tests/bundle_activity_content.rs` because kart and encounter saves
  embed their content hashes.
- `reference/`: the 46 files listed by
  `lab/references/rust-gateway-reference/manifest.json`; the builder checks
  each source length and SHA-256 before copying.
- `startup/startup.json`: the typed startup snapshot from
  `lab/content/startup-snapshot/startup.json`. The snapshot exporter omits
  startup message 07 and has no fields for a player's name, GUID, inventory or
  chat; the gateway keeps message 07 on its character-specific path. It still
  carries capture-derived reference-character stats and journal identifiers,
  so it is a private packet template rather than a player save. This boundary
  is recorded in `notes/Research/Startup Integration.md` and implemented by
  `startup_snapshot.rs` / `gateway.rs`.
- `login-resources/`: the seven character-creation tables from the extracted
  client `Resources` tree. `client-locale/`: the two English locale files from
  `lab/references/opening-native-lobby-20261009T163603Z/locale`. These are private
  client data, not newly recovered behavior.
- `terrain/`: `zone_*.hgt` grids from `lab/terrain`.
- `server/`: a newly written schema-3 empty account store and a random 32-byte
  server identifier. Neither is copied from an existing server or player.
- `README.md` and `manifest.json`: startup instructions and integrity metadata.

Activity content is required by the bundle gate. `check_activity_content` validates
the ten files above against bounds that mirror the Rust loaders (encounter template
hash, one owned kart vehicle and its 41/114 frames, the five-table lobby body,
game/profile bindings, a race schedule that reaches `countdown: 0` (the T028 loader
gate), board shapes, node tables and fishing spots). A bundle with a missing or
mismatched activity file is refused. `kart_gates.json`
(`{"schema_version": 1, "npcs": [9000000273, 9000000039], "reward_cooldown_s": 300}`)
is now committed; the builder's generated copy remains a fallback for old trees.
**(R)** Chase Wheeler and Cale Yardbird were picked from wiki and map-marker evidence;
the original NPC-to-kart mapping is not recovered.

Optional runtime content (W159, 2026-10-16). The gateway reads these from `content/`
when present and runs without them, so the builder never fails on a missing one and
reports each as included or absent (`Runtime content:` line). `consumables.json`
(item-bar potion and food tuning, `item_bar.rs`; a built-in table applies without it),
`power_ups.json` (combat power-up drop tables, `power_ups.rs`),
`crafting_stations.json` (world cooking, smelting and anvil models, `recipe_book.rs`)
and `job_trainers.json` are tracked repository content and enter through the
committed-`HEAD` copy, so the same hashing and manifest checks cover them. The
builder prints a `WARNING` for uncommitted `data/content` edits.
Warpstone and quest-journal data need no entry: quest-journal text is `journal-text.json`
and `quest-journal.bin`, already in the private overlay.

The round70 content is a playable package, not a claim that all of its rules are
historically exact. Its per-file provenance distinguishes client-backed data,
Legacy-derived starting material and reconstructed server policy. `opening.json`
and the activity files retain those annotations. The client login resource tables
are from `lab/working/OSFR.Client/OSFR Client/Resources`; packet templates are
from the hash-verified gateway reference above; journal text and terrain retain
the listed lab origins. The local smoke probe follows the current Rust
implementation in `prototype/osfr-tools/src/bin/osfr-login.rs`, `login.rs`, and
`identity.rs`. That check does not independently establish original-client
protocol behavior.

`quest-journal.bin` is included only as the runtime root-packet template named
and hash-bound by the bundled `content/quests.json`. It is private client-derived
data. The active account store, `identity.json`, `character.json`, and
`characters/` saves in the round70 package are excluded.

## Build and run

From the repository root:

```powershell
python -I scripts/bundle/make_bundle.py
python -I scripts/bundle/check_bundle.py --bundle ..\lab\bundles\playtest-<UTC timestamp>
```

Setting `OSFR_BUNDLE_ROOT` moves both the bundle folder and the checker's scratch
runtime (for example to `lab/build/<worker>/`) so a trial build never touches
`lab/bundles/`.

The generated README contains the complete private-toolchain build, fresh
runtime directory, account provisioning, login and gateway commands. The login
and gateway share a runtime account store and handoff key, bind to loopback, and
use a client protocol key from `OSFR_CLIENT_LOGIN_KEY_FILE`. The bundle never
contains that key or a player's private key. The checker verifies the manifest,
starts login and gateway on loopback ports, creates one disposable character,
runs one `osfr-bot` through gateway login and a short persisted walk, stops its
exact child processes, and deletes its temporary runtime folder. The login probe
creates a random protocol key only inside that temporary runtime; no host
account or key is used by the probe or copied into the bundle. The checker sets
`OSFR_GATEWAY_STARTUP_SNAPSHOT` to the bundled typed snapshot. This local
single-session path does not test account-store HMAC handoff or the launcher
credential broker.

`check_bundle.py --verify-only` checks file sizes and SHA-256 values, enforces
the empty account store and sensitive-path exclusions, then compares every
payload digest against files under `lab/state`, `lab/keys`, `lab/accounts`,
active-package character saves and root identity/account files,
sensitive-named files in `lab/run(s)`, and the preserved client login key path.
This is a bounded scan of the project's documented private roots; it does not
search unrelated user folders. To verify tamper rejection, alter one payload in
a disposable copy and require `check_bundle.py --verify-only` to fail.

## Manifest and W121

`manifest.json` uses W121's parser shape: `{"version":1,"files":[...]}` with
each file entry containing `path`, `size`, and lowercase `sha256`. Paths are
sorted, relative, launcher-safe ASCII names. It lists every bundle file except
itself; the nested gateway `reference/manifest.json` is included as a payload.
The playtest stamp is the folder name, and the committed source revision is in
the README. The launcher trusts an operator-pinned manifest SHA-256, so the
bundle's manifest must receive a trusted pin before launcher use. The file
hashes detect content changes but do not sign the bundle or identify its author.

## Distribution boundary

Keep bundles under `lab/bundles/`. Do not publish or commit them. They contain
private client-derived packet templates, localized text and character-creator
tables, the captured world-entry snapshot, terrain data, and activity content
that includes Legacy-derived material. Public distribution requires a separate
content and licensing review.
Never add an existing account store, character identity, player save, client
login key, gateway handoff key, or private signing key.

## Gate boundary

The checker validates this repository's Rust login, character creation, bot
gateway login and movement using the packaged inputs. It is a local integration
smoke test. Native-client acceptance and full historical content fidelity remain
separate gates in the Build Plan and Build Ledger.
