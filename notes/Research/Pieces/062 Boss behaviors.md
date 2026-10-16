# 062 Boss behaviors

Status: ready for review, 2026-10-09 05:48 UTC. This piece recovers **one original boss's phases,
abilities and authored attack timing**: the `Necronomicus Unleashed` encounter (original encounter
432, "Necronomicus' Lair"). It is evidence plus a checker plus a typed module. It is **not** a
playable claim, grants nothing, and presents no balance value as historical.

Owned paths: `prototype/osfr-tools/src/restoration/piece_062.rs`,
`prototype/osfr-tools/tests/piece_062.rs`, `scripts/pieces/piece_062.py`,
`tests/pieces/test_piece_062.py`, this note and
`lab/reports/piece-062-20261009T0533Z/`. No shared `lib.rs`, gameplay, gateway, Cargo manifest,
launcher, content, state or native UI file was touched.

## Recovered original facts

Identity. The original client's own `Resources/Models.txt` maps model id `4220` to
`necronomicus_m.adr`; the community NPC mirror `Sanctuary/src/Resources/Npcs.json` row `3283`
carries name id `440729`, model id `4220` and the same file name, and the preserved locale export
resolves `440729` to `Necronomicus`. The encounter is named by locale `5100662`,
`Encounter 432 - Necronomicus' Lair`; locale `442596` names the battle `Necronomicus Unleashed`,
which the read-only wiki `Battles` page lists as difficulty 5 ("Requires entry with a combat job at
least level 13 or higher"). Locale `442600`/`442598` carry the separate Awakened/Overworld
variant ("recommended for a full group of level 20 players").

Objectives. Locale `5100663` `Defeat the necromancer!`, `5100664` `Defeat Necronomicus!`,
`5100666`/`5100667` `Destroy the Zombie Portals`, and the eight per-portal objectives
`Destroy Zombie Portal 1..8` (`5100669`-`5100683`, odd ids, the even ids are blank). Three
`Necronomicus Buff Package 1/2/3` rows (`5100686`/`5100688`/`5100690`) sit beside the dev text
`[dev]Buff Package of Region 2 Boss` (`5100687`).

Phases. The `Necronomicus` wiki page documents the fight in three stages, and the original locale
carries independent evidence for each boundary:

| Phase | Original / wiki evidence |
|---|---|
| 1. skeletal form | "Necronomicus is a fairy NPC boss"; "Necronomicus' skeletal form"; "he will use magic and attack you" |
| 2. skeleton pieces + zombie portal(s) | "When Necronomicus is defeated, the skeleton pieces will appear on the ground and zombie portal will appear which spawn necrowart zombies to attack you"; locale `5100650` "Have you met my friends?" (wiki: "when a zombie portal appeared") |
| 3. powered skeletal form | "When the zombie portal is destroyed, Necronomicus make his own form which its appearance is a powered skeletal creature"; "he can smack the ground which make it shake"; locale `5100995` `[dev] Necronomicus Phase 3 Basic Attack`; locale `5100653` "My power is eternal!" (wiki: "when his own form is completed") |
| terminal | "Once defeated, Necronomicus will lose his form and he will also been beaten." |

Abilities and animation binding. Locale names and descriptions: Ground Pound
(`5100996`/`5100997`), Hungering Darkness (`5100998`/`5100999`), Heart of Darkness
(`5101000`/`5101001`), Necrotic Gloamfire (`5100693`/`5100694`, "signature magic blast"),
Necrotic Gloamfire Splash (`5100695`/`5100696`), basic-attack rows (`5100994`, `5100995`,
`5101009`), `Necronomicus Cave-in` (`5101015`/`5101016`, "triggered from N's Ground Pound
ability") and the Overworld variants (`5101010`-`5101014`).

The original actor definition `necronomicus_m.adr` (pack `AssetsW_001.pack` index 3350) binds 31
animation slots and gives each attack slot an authored `ap_contact` / `ap_interrupt` pair. Both
the pinned community reader `edenfps/fr-adr-toolkit` (revision
`5acfdf8d6006fa6f43d1d3605f076a6be7393e72`) and the checker's narrow reader return the same
slots and clips:

| Slot | Clip | Effect binding | `ap_contact` | `ap_interrupt` |
|---|---|---|---|---|
| `com_h2h_attack_01` | `necronomicus_com_h2h_attack_clap.gr2` | - | 0.7036 | 0.8341 |
| `com_h2h_attack_02` | `necronomicus_com_h2h_attack_flick.gr2` | - | 0.7820 | (none) |
| `com_h2h_attack_03` | `necronomicus_com_h2h_attack_kick.gr2` | - | 1.0253 | 1.2724 |
| `com_h2h_attack_04` | `necronomicus_com_h2h_attack_headbutt.gr2` | - | 0.5998 | 0.7583 |
| `com_cast_01` | `necronomicus_com_ground_pound.gr2` | `ground_pound` / `ground_pound_effect` | 1.1955 | 1.4593 |
| `com_cast_02` | `necronomicus_com_360_attack.gr2` | `Head_trail` / `head_particles` | 1.5978 | 3.1642 |
| `com_cast_03` | `necronomicus_com_chest_laser.gr2` | `magic_beam` / `magic_beam_shot` | 3.0953 | 4.5277 |

So the three documented specials map onto the actor's cast slots by the actor's own data: Ground
Pound -> `com_cast_01` (the clip is literally named `ground_pound` and the slot's effect binding is
`ground_pound_effect`), Hungering Darkness -> `com_cast_02` (its effect sequence is `Head_trail` /
`head_particles`, matching "removes his head"), Heart of Darkness -> `com_cast_03` (the chest
laser plus `magic_beam` / `magic_beam_shot`, matching "the great skeleton's chest opens"). The
same file additionally records `com_cast_01` -> effect `necronomicus_ground_shake`, cue
`ground_shake`, with two raw `f32` values (1.1765, 1.5228) whose meaning is unrecovered.

Timing outside the actor. `necronomicus_pixie_gloam_smoke.xml` (spawn smoke) carries
`EndTime="3.0"`; `necronomicus_chest_beam.xml` carries `EndTime="3"` and a `Hit_Flash` instance at
`StartEmission Time=".50"`. Three original barks ship in the packs, and their MPEG frame durations
measure 1.392 s (`DX_NECRONOMICUS_BACK.mp3`), 2.136 s (`DX_NECRONOMICUS_NOT_SO_FAST.mp3`) and
3.552 s (`DX_NECRONOMICUS_POWER.mp3`). The actor definition names three dialogue cues
(`DIA_Necronomicus_Friends`, `DIA_Necronomicus_Not_So_Fast`, `DIA_Necronomicus_Power_Eternal`)
that resolve to locales `5100650`, `5100652` and `5100653`; **which cue plays which recording was
not recovered.**

Assets. 29 of the 31 actor slots resolve to a supplied pack row whose payload matches the
catalogue size and CRC32. `necronomicus_spawn_stand_loop_02.gr2` and
`necronomicus_spawn_stand_loop_03.gr2` (slots `amb_loop_01_fidget_01/02`) have no supplied pack
row and were not fabricated. The lair zone is present as `mv_necronomicus.gzne` and
`mv_necronomicusAreas.xml` (an ambient-audio sphere at `193,-49,252`, radius 35 - an ambience
volume, not an arena bound).

## Community mapping (never merged into the original section)

* `Npcs.json` row 3283 and `Models.txt` row 4220 are community mirrors of original tables; both
  agree with the preserved locale and the supplied actor definition.
* The wiki `Necronomicus`, `Necrowart Zombie` and `Battles` pages are community documentation. The
  phase sentences above are wiki text; the phase *existence* is independently corroborated by
  locale `5100994`/`5100995`, `5100686`-`5100690` and the eight portal objectives.
* GitHub code search for `necronomicus` and for `Necronomicus Unleashed boss abilities` returns no
  Free Realms boss AI, phase script or encounter data - only locale tooling and unrelated
  projects. No community encounter script was found to reuse.

## Reconstructed boundary

`piece_062.rs` implements the recovered constants, the ability/timing tables and one pure helper:
`advance(phase, event)` re-derives the documented phase order and refuses any out-of-order event.
That helper is **reconstructed ordering**, not the original scheduler; nothing drives a live
encounter, spawns portals, applies damage or grants loot.

Two interpretations are labelled rather than asserted:

* the `ap_contact` / `ap_interrupt` values have no unit byte in the file; "seconds inside the
  clip" is the working interpretation (every value lies inside its clip and the seven values
  increase with clip length), and the server-side schedule they fed was not recovered;
* the actor's per-slot playback float is named `speed` by the pinned community reader; its
  values are exact 30 Hz frame-count fractions (e.g. `4.633334` = 139/30), which suggests a clip
  duration, but the module records it as a raw scalar and does not claim a duration.

## Explicit gaps

* server cooldowns and cast scheduling: not recovered (only the authored animation phase times);
* phase health thresholds and any transition condition beyond the documented order: not recovered;
* portal entity/actor ids and the portal spawn table: not recovered (only the eight objective
  strings);
* which bark cue plays which `DX_NECRONOMICUS_*` recording: not recovered;
* GR2 clip durations: the clips were not decoded, and a custom Granny reader was not written
  (community `opengr2` is the named next reader if a duration is ever needed);
* the meaning of the two `necronomicus_ground_shake` floats: not recovered;
* `necronomicus_spawn_stand_loop_02/03.gr2`: no supplied pack row;
* ability-to-phase assignment for the basic attack, Necrotic Gloamfire and its splash: not
  recovered, so those rows keep an empty slot;
* native animation acceptance: not performed by this item.

## Verification

```
& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_062
  RED  (module absent): error: couldn't find file `tests\..\src\restoration\piece_062.rs`
  GREEN: 8 passed; 0 failed; 0 ignored
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_062.py"
  RED  (checker absent): Ran 11 tests, FAILED (failures=10, errors=1)
  GREEN: Ran 11 tests, OK
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_062.py --output <report>\piece-062-evidence.json
  piece 062 evidence ok: Necronomicus encounter 432, 3 phases, 8 abilities, 7 timed attack slots, 29/31 clips resolved
& '..\lab\tools\rustup\toolchains\stable-x86_64-pc-windows-gnu\bin\rustfmt.exe' --check --edition 2024 <both owned Rust files>
  clean
```

The checker fails closed: locale text drift, a catalogue size change on a boss clip, a wiki phase
sentence change, a flipped gap boolean, a doubled bark recording, a missing actor definition or a
missing wiki corpus each return exit code 2 with a named check. Raw evidence (RED/GREEN logs,
`piece-062-evidence.json`, `community-reader-adr.txt`, `rustfmt-check.log`) is under
`lab/reports/piece-062-20261009T0533Z/`. Source hashes the checker records: locale export
`f1f22a30..e62844`, pack catalogue `12bfd0f8..45b1c9`, actor definition `f939186d..664b88`,
client `Models.txt` `0bad5a15..5076be`, `Npcs.json` `b51d2d92..c0b38f`, wiki corpus
`b9d7b9f1..fcd911`.

## Next gates

1. Lead integration: register `pub mod piece_062;` under `src/restoration/` and drop the
   `#[path]` include. This item owns no live encounter data; encounter 432 is a name, not a
   scheduled spawn.
2. Independent review of the ability/slot mapping and the two labelled interpretations above.
3. Native acceptance (lead-owned, shared client): enter the lair and observe, in order, the
   skeletal form, the skeleton-pieces/portal stage and the re-formed powered form with
   `necronomicus_com_ground_pound.gr2` (`com_cast_01`), `necronomicus_com_360_attack.gr2`
   (`com_cast_02`) and `necronomicus_com_chest_laser.gr2` (`com_cast_03`). The 2.136 s
   `DX_NECRONOMICUS_NOT_SO_FAST` and 3.552 s `DX_NECRONOMICUS_POWER` barks are the expected
   transition beats.
4. Cross-piece: 054 (melee) should consume the recovered contact/interrupt phase times instead of
   inventing attack windows; 051/059/061 own the enemy catalogue, dungeon definitions and
   wave/gate data that would schedule these abilities; 065 owns the full native replay.

Cross-links: [[../../Design/MMO Worker Contract]], [[../../MMO Work Queue]],
[[../Starting Dungeon Audit]].
