---
type: research
updated: 2026-10-15
task: W85
verdict: PARTLY PROVEN
---

# Job Basic Attacks

Ninja (2), Warrior (32) and Wizard (12) basic toolbar rows are staged. Each killed a Gloamling through the real scratch gateway and `osfr-bot`. This proves the existing Rust route with the new data. It does not prove original ability IDs, historical balance, native visuals or weapon delivery. The active native campaign was not changed.

## Files and integration boundary

- `prototype/osfr-tools/data/content/opening-abilities.json`: **three additive rows only**, in the existing private `OpeningAbilities` schema. Only profiles 2, 32 and 12 are owned here.
- `scripts/bot/w85-ninja.bot`, `w85-warrior.bot`, `w85-wizard.bot`: repeatable damage/removal checks; no `realm win`, rewards, content edits or weapon grants.
- This note: provenance, runtime rules, reproduction, checks and gaps.

The Ledger's gap is real, but the runtime path matters. `gameplay.rs:275` calls `OpeningAbilities::load(directory)` against the **private `--content` campaign**, whose `opening-abilities.json` contains only 43/11/35. `gameplay.rs:478-481` separately loads the repository's `job_abilities.json`; `opening_abilities.rs:130-165` permits only consecutive extra slots **1..7**, on a job already having a basic. Adding slot 0 there would be invalid. No new loader, Rust module or dispatch was built.

**Lead integration required:** merge the three staged `basics` by `profile_id` into a newly frozen active campaign's `opening-abilities.json`, preserving profiles 43/11/35 and their provenance. Restart that campaign: `realm reload abilities` reloads extra skills, not this private basic map. Merely adding this repository JSON does not enable the jobs in an existing campaign. The tested six-row merged artifact is `lab/build/w85/content/opening-abilities.json` (SHA-256 `8a989d65ced6399ade78427d8f65ead2b20aa7579c85f7d23941be45a587a83b`). This is a concrete candidate for lead review, not an integrated milestone. Shared Plan/Ledger/MODLOG files were outside worker ownership.

Graph search found `OpeningAbilities::load_job_abilities` called by `Gameplay::load`, `reload_content` and the job-level test. Coverage check reported changed metadata for gameplay/encounter; their relevant source was read directly before relying on the route.

## Client facts and reference bindings

Client resources inspected under `lab/runs/client-baseline-20261014T203930Z/client/`; the job-id table was independently read in `lab/working/OSFR.Client/OSFR Client/Resources/ItemFilterOptions.txt`.

| Job | Client job-id row | Selected basic name | Client locale line | Icon / Images.txt line | Weapon-name id / locale line |
|---|---|---|---|---|---|
| Ninja 2 | ItemFilterOptions.txt:18, name 3100 | Lightning Strike, name 20217 | locale/en_us_data.dat:40008 | 14813 / 10898 | 131975 / 95894 |
| Warrior 32 | ItemFilterOptions.txt:22, name 4664 | Sweeping Slash, name 420252 | locale/en_us_data.dat:69465 | 14104 / 10189 | 13699 / 106362 |
| Wizard 12 | ItemFilterOptions.txt:23, name 2280 | Zap, name 17071 | locale/en_us_data.dat:48499 | 14434 / 10519 | 11056 / 53451 |

These are actual `Global.Text.<id>` lookups, not raw-hash substitutions. Icon 14813 is the wooden Ninja training sword, 14104 a Warrior battleaxe, 14434 a Wizard wand. Those image records prove the image exists; **the chosen name/icon/weapon binding is reference data**, not a recovered retail assignment. Weapon names resolve to Ninja's Training Sword of 1000 Storms, Student Warrior Cudgel and Student Wizard Wand. The shipped tutorial at locale hash 819381112, line 21618, explains that abilities depend on the equipped weapon.

Native ability IDs are **not recovered**. The reviewed guide and `Ability System.md` state that definitions arrive from the server. Read-only native-map checks:

```text
python -I scripts/native_map/nmq.py brief 00a32930 00a34d10 00a35150 00a2a910
python -I scripts/native_map/nmq.py label 00a32930
python -I scripts/native_map/nmq.py dis 00a32930 80
python -I scripts/native_map/nmq.py show 00a34d10
```

- `00a32930` is accepted/unknown. Bytes `00a32937-00a32962` show a bounded dword read into object+8, followed by bounded byte reads. This supports parsing a supplied definition, not a constant job ability ID.
- `00a34d10` (likely) looks up a definition by its supplied ID, queues an unknown ID and sends a request via `00a2d270`/`00a34410`; the inspected pseudocode has no job-specific constant table.
- `00a35150` is the mapped ability-use sender; `00a2a910` reads the processed-hit payload. Field interpretation follows the reviewed combat guide; no new speculative interpretation was added.

The **compatibility definition id 4895** is reused exactly as for all three working basics. Its original job-specific identity remains unproven. Community combat row IDs below are provenance keys and are **not sent as retail ability IDs**:

| Job | Sanctuary CombatJobs.json line | CombatAbilities.json line / community row | Starter definition / Legacy ClientItemDefinitions.json line |
|---|---|---|---|
| Ninja | 27 | 65 / 2154 | 75091 / 1206973 |
| Warrior | 101 | 159 / 5011 | 29963 / 617951 |
| Wizard | 77 | 124 / 4011 | 4914 / 92999 |

Pinned data-only references, independently checked with `git rev-parse HEAD`:

- Sanctuary `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`, AGPL-3.0, https://github.com/Open-Source-Free-Realms/Sanctuary. `src/Resources/CombatJobs.json` SHA-256 `2acec6764bb269797f0ac80335057f17cda7d917d36eb262eba2fb7a30134390`; `CombatAbilities.json` `ecf3d2d6aa92df6a2bdbe6126874a664c862be167246a977368ffd71849cec7d`.
- FreeRealms-Legacy `9e082025391177d5a33ee18c714ddd06b457a695`, AGPL-3.0, https://github.com/raisingkaines/FreeRealms-Legacy. `src/Resources/ClientItemDefinitions.json` SHA-256 `40a3cc79cfd8adfd50dc689e58de18dc14e21bedd73993b6eab0a43d3d95f025`.

No community source code was copied. No raw resources or decompiled code were added to the repository.

## Animation, timing and final-era rules

**Recovered client animation facts:** `Resources/WieldTypeAnimationMappings.txt:11,14,12` maps Ninja/Warrior/Wizard wield types 9/12/10 to group **1099**. `CodeAnimationMappings.txt:148` names it `com_swing`; `AnimationGroups.xml:635` gives fallback 1000. `AnimationTransitionRules.xml:154` uses one combat play, speed 1.0, zero delay and 0.2 s easing; line 52 returns to standing with 0.6 s easing. **Animation clip length, hit-frame time and original server recast are not recovered.**

**Reconstructed (R), reused unchanged:** `encounter.rs:2976` already defines Ninja range/base 3 m/100, Warrior 3 m/110, Wizard 20 m/130. `encounter.rs:2009` enforces 600 ms between hits; line 2063 sends swing 1099. All three resolve one immediate hostile hit, with zero energy cost. These are the existing supported basic rules, not historical values. The reference Warrior row says AoE, but this change does not claim or add AoE. Wizard has no new projectile/cast effect.

Final-era tier scaling is already routed through `sync_player_level` to `EncounterEngine::set_player_job` (`gameplay.rs:4646`, `encounter.rs:1166`). It multiplies base damage by **1.75^(tier-1)**. `weapon_tier` (`encounter.rs:35`) selects tiers at levels **1, 4, 5, 8, 12**. Rounding, level-as-tier selection, base values and job crit policy remain (R). Reusing this path avoids a competing balance table:

| Job | Level 1 / tier 1 | Level 4 / tier 2 | Level 5 / tier 3 | Level 8 / tier 4 | Level 12 / tier 5 |
|---|---:|---:|---:|---:|---:|
| Ninja | 100 | 175 | 306 | 536 | 938 |
| Warrior | 110 | 193 | 337 | 590 | 1032 |
| Wizard | 130 | 228 | 398 | 697 | 1219 |

Values above are calculated from existing policy, before crits and target-health clamping. The bot gates use level 4: one hit exceeds the Gloamling's reconstructed 150 health; the server clamps damage dealt to 150. This is an integration check, not a retail balance measurement.

## Weapon dependency and security review

[[Job Starter Outfits]] (W82) independently identifies the same proposed weapons. Its private starter-list artifact is `lab/reports/job-outfits-20261015/starter_items.json`. In frozen `round69-native-20261015T224835Z/content/inventory.json`, Ninja 75091 already has a definition; Wizard 4914 and Warrior 29963 need W84 catalogue coverage. All three still need the W82 starter grant/equip integration. Those selections are (R), not recovered default policy. No inventory rows or saves were edited by W85.

The scratch campaign explicitly unlocked profiles 2/32/12 in its copied `profiles.json` defaults. It had **no starter_items entries for those jobs**, and the bots granted/equipped nothing. Therefore the passing runs prove the server's current unarmed compatibility route; they do **not** prove the weapon, wield state or animation can render in the native client.

Applied `secure-code-guardian` and `security-reviewer` in scope:

- Threat model: hostile ability requests must not supply ownership, damage, range, position authority or developer privileges. Test helpers must not affect native saves or expose dev tools.
- Automated scoped checks: JSON has exactly the three owned profiles and the six existing integer fields per basic; bot scripts have no secrets, file/network configuration, reward cheats or content mutation commands. Content checker passed. Existing hotbar validation tests cover invalid fields, duplicate profiles and size bounds.
- Manual review: only trusted JSON and bot input files changed. Existing loader bounds remain 64 KiB/16 basics, with field/duplicate validation. Existing gameplay requires an active supported basic; encounter parsing validates bar/slot/mode/length and enforces server target health, range and repeat time. No authorization code or listener settings changed. Every scratch gateway bound `127.0.0.1:0`, enabled dev tools only there, set `--studio-port 0` and expired after 10 seconds. All scratch helpers exited.
- **New code security findings: none. Existing gameplay-integrity gap (Medium):** attacks do not check the equipped weapon; this was reproduced by the unarmed bots. Starter delivery alone does not add that check. Weapon/line-of-sight enforcement needs lead-owned runtime work if required for playtest policy. No unsupported claim that W85 fixed it.

No new dependency, global tool, native-client session or external message was used. Reverse-engineering skill supplied the byte/provenance discipline. Game recon and asset readers were unnecessary because the engine and text resource formats were already known; no asset format was implemented.

## Reproduction and gates

Commands run from `project`. Scratch outputs only under `lab/build/w85`; `CARGO_TARGET_DIR` was that directory throughout.

The test campaign copied `lab/references/opening-anvil-placement-v1/content` to `lab/build/w85/baseline-content`; only the copied `profiles.json` default unlocked list gained 2/12/32. A second copy, `lab/build/w85/content`, appended the three authored basics to the original 43/11/35 list and retained its provenance. Source campaigns and native saves were untouched.

There is **no `realm job` command in this checkout**. Each script uses the existing `26/13` class-switch request, asserts `38/21`, `36/13` and `36/5`, then uses `realm level` and `realm battle`. An initial `realm goto` attempt was refused inside the opening cave by the existing guard (`gameplay.rs:3349`); scripts use a small ordinary `move` instead. No guard was weakened.

Gateway command per job (new state filename and ephemeral port each time):

```powershell
$env:CARGO_TARGET_DIR='%USERPROFILE%\Documents\Projects\CartoonMMO\lab\build\w85'
& ../lab/build/w85/debug/osfr-gateway.exe 127.0.0.1:0 --reference ../lab/references/rust-gateway-reference --content ../lab/build/w85/content --state ../lab/build/w85/<job>-move-state.json --dev-tools --studio-port 0 --seconds 10
& ../lab/build/w85/debug/osfr-bot.exe --gateway <listening-address> --reference ../lab/references/rust-gateway-reference --expect-timeout 2500 --script scripts/bot/w85-<job>.bot
```

Actual bot log tails:

```text
Ninja — gateway 127.0.0.1:64400, PID 22696, exit 0:
[bot1   2357ms] <- 32/7 len=50
[bot1   2357ms] <- 35/3 len=31
[bot1   2398ms] ok expect-text "dealt 150"
[bot1] PASS (163 messages received)

Warrior — gateway 127.0.0.1:57007, PID 27444, exit 0:
[bot1   2171ms] <- 32/7 len=50
[bot1   2171ms] <- 35/3 len=31
[bot1   2211ms] ok expect-text "dealt 150"
[bot1] PASS (162 messages received)

Wizard — gateway 127.0.0.1:58392, PID 13432, exit 0:
[bot1   2197ms] <- 32/7 len=50
[bot1   2197ms] <- 35/3 len=31
[bot1   2224ms] ok expect-text "dealt 150"
[bot1] PASS (163 messages received)
```

Full logs: `lab/build/w85/{ninja,warrior,wizard}-bot.log`; gateway logs/errors alongside them. Independent saved-state inspection confirmed active profiles 2/32/12, position `[291,28,289]`, first Gloamling GUID `5715144128442986753` health **0** in all three `<job>-move-state.json` files. No `win` cheat was used.

Canary: the same Ninja request with the new rows absent, same job, level, position and enemy, and only the unavailable-definition expectation removed:

```text
[bot1   2319ms] -> ability 36/10 slot=0 guid=5715144128442986753 len=31
[bot1] FAIL: expect 32/7: timed out after 1500 ms
canary profile 2 position [291.0, 28.0, 289.0] enemy_health 150
CANARY PASS: rows absent, same in-range request did no damage; rows present, all three jobs killed enemy.
```

Preserved as `canary.bot`, `canary-bot.log`, `canary-state.json` in the same build directory. A first helper assertion checked stderr instead of stdout and failed; the saved combined log and state were subsequently checked successfully. That helper failure did not change the bot result.

```text
python -I scripts/content_check.py --out ../lab/build/w85/content-before.json
== 0 errors, 17276 warnings across 133 files
python -I scripts/content_check.py --out ../lab/build/w85/content-after.json
== 0 errors, 17276 warnings across 134 files
```

Warnings are unchanged; none was added by the attack rows. `--out` kept generated reports out of shared Studio files.

```text
.\scripts\cargo.ps1 test --manifest-path prototype/osfr-tools/Cargo.toml --test abilities --test opening_abilities
test result: ok. 8 passed; 0 failed; 0 ignored
test result: ok. 4 passed; 0 failed; 1 ignored
```

That Cargo run built the scratch binaries and passed before concurrent auth changes. After the data edits, the same compiled test executables were rerun from `prototype/osfr-tools` and again passed **8/8 and 4/4, one private oracle ignored**. Logs: `abilities-tests.log`, `opening_abilities-tests.log`. An initial direct invocation from the repository root missed the existing relative Medic fixture; rerunning from the crate directory fixed that invocation error.

**Fresh current-checkout Cargo rerun blocked by concurrent W83 tooling changes:** first `ed25519-dalek` was unavailable offline; after it was cached, `getrandom` failed locating `dlltool.exe`. Adding the already installed GNU self-contained bin to the process PATH found dlltool, but it failed `CreateProcess` while generating the bcryptprimitives import library. No shared manifest, wrapper or tool installation was changed by W85. A scoped `--lib player_health_and_damage_scale_with_job_level` attempt was also blocked at dependency resolution. Thus the existing built ability tests pass, but a new build against the now-changing shared checkout is **not proven**.

Tested binaries (debug builds under the owned build directory):

- `osfr-bot.exe`: SHA-256 `8b492e5c48100fd0eead9b9efcee8514df876cdd198c4391e20a4888844bb68b`.
- `osfr-gateway.exe`: SHA-256 `382614974cbf4acdc6cc1ca9430c69362df85ba5493eada61a88e4da85d727ae`.

## Remaining acceptance

**PARTLY PROVEN:** all three data rows load and kill through the real scratch route; unchanged content checker and existing compiled ability tests pass; the absence canary detects the missing support. Lead integration into a frozen runtime campaign, a fresh build with concurrent auth/toolchain work resolved, W82/W84 weapon delivery and native-client visual/weapon checks remain open. Original ability IDs, weapon-specific AoE/projectile behaviour, timing, range and historical balance remain unrecovered. No complete-combat or historical-fidelity claim is made.

Ponytail: reused the basic loader, toolbar serializer, class switch, battle/attack/scaling route, content checker and bot client. Built no Rust, dependencies, alternate combat engine, loader or new balance system.
