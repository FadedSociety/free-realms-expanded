# 037 Medic progression

Status: ready for review, 2026-10-09 04:05 UTC. This piece recovers the **Medic job identity, its
healing ability text and ranks, the energy resource and its refusals, the knock-out rules and the
revive window contract**. It is evidence plus a checker plus a small typed module; it is not a
playable claim and no balance value is presented as historical.

Owned paths: `prototype/osfr-tools/src/restoration/piece_037.rs`,
`prototype/osfr-tools/tests/piece_037.rs`, `scripts/pieces/piece_037.py`,
`tests/pieces/test_piece_037.py`, this note and `lab/reports/piece-037-20261009T035019Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Identity. The client declares wield type `14` as `Medic` in
`Resources/WieldTypeAnimationMappings.txt` (`14^Medic^1099^0`). The job strings are client locale
2278 `Medic`, 2279 "Medics use medicine to keep allies healthy as well as debilitate opponents." and
hint 384185 "If you've always wanted to take care of others, Nurse Naia will get you started." The
Medic weapon animations all map to the shared melee group `1099`, so the job is melee in the client
animation table.

Resources. `Resources/CharacterStatDefinitions.txt` gives stat 1 -> locale 22485 `Health` and stat 5
-> locale 22489 `Energy`. `Resources/AbilityEffectTypeSettings.txt` splits the effect vocabulary:
type `3` carries the ranged heal/restore tooltips (id 3 -> 92026 "Heals you for
`Param1TunedLow` - `Param1TunedHigh` health.", id 4 -> 2492 "Restores … energy."), type `1` carries
damage (id 38 -> 2983 "Deals `Param1TunedLow` damage.") and type `2` carries the flat
`Param2TunedLow` stat text. Two refusals are recovered verbatim: locale 285 "You don't have enough
energy for that ability!" and locale 280 "You can't use abilities when you are knocked out!".
Locale 37466 documents the knock-out itself ("Once you're knocked out, you'll be down for a few
seconds … wait until you can get back up"), and locale 4035 records a Medic-specific proc, "When the
medic knocks out an opponent they gain 2 energy."

Abilities. Incision is locale 420174 with description 422746, "Strikes with surgical precision,
damaging all opponents in front of you." Triage is 420186 (repeated at 442456 for the awakened
weapons) with description 422736/442468, "Sorts out friend from foe with surgical precision, healing
you and your group and damaging all nearby opponents." — this single sentence is the recovered
friendly/opponent split for the job. Target Vitals is 3810 with tier label 420497 " Medic Tier 1 -
Ultra Ability - Target Vitals" and description 422737 (damage plus a defence debuff). The job's own
heal is First Aid, locale 3784, with five recovered rank texts (3813 and 47279-47282):

| Rank | Text id | Heal |
|---|---|---|
| 1 | 3813 | 250 |
| 2 | 47279 | 281 |
| 3 | 47280 | 313 |
| 4 | 47281 | 344 |
| 5 | 47282 | 375 |

Every rank reads "Heals yourself and any ally near you for N health", ranks 1-4 add "Next rank:
Heals for M health.", and rank 5 carries no next rank, so the ladder is closed at five entries with
no invented sixth. Shock Paddles is locale 3800; the level-15 lesson text 73908 says the paddles
"have two purposes, shocking your opponents, and reviving your allies", and 73914 continues "let's
assume that Clover is your ally, and has been knocked out…". Descriptions 3815 ("Causes 75 damage to
an opponent and two other opponents near that opponent.") and 3812 (Vitamins) sit in the same block.

Healing effects. Triage's recovered client effects are 16162 `PFX_heal_red_hands_cast_starburst`
(cast) and 16161 `PFX_heal_red_explosion_lg_AOE` (hit) in
`Resources/ActorCompositeEffectDefinitions.xml`. The same table carries 4002 `PFX_magic-resurrect`;
locale 22111/22112 describes the post-revival ability that is "placed on the PCs after they revive"
and states it has "2 effects", and locale 20463 describes the (deprecated) Resurrection Scroll that
"resurrects a fallen ally".

Revive window. The client binds `Main.wndRespawn` to `UI\revive_window.swf` in
`UI/UiModules/Main/wndRespawn.xml`. The Lua `RespawnWindow:DisplayRespawn(timer, isOverworld,
reviveCost)` shows a coin dialog only when `isOverworld == 1` — the paid button string is built from
`\nRevive here:\n %1 coins` — and otherwise calls `StartTimer(timer)`. Its events are `respawn` ->
`Ui.RevivePlayer()` (logged with `Ui.SetWallOfData("RespawnWindow", "OnNonPaidRevive")`) and
`OnPaidRevive` -> `Ui.OnPaidRevive(b)`. The decompiled original `revive_window.gfx`
(`lab/reports/piece-037-20261009T035019Z/revive-window/`, ffdec 26.3.0, pinned command below) adds
the exact contract:

* `setCoinAmount_lua(n)` stores `m_coinWallet` and `setReviveCost_lua(n)` stores `m_reviveCost`;
* `showTwoButtons_lua` runs `if(m_coinWallet < m_reviveCost)` and calls
  `twoButtons_mc.paidRevive_mc.enable(false)` in that branch, `enable(true)` plus
  `addListener(this)` otherwise;
* the alternative button always reads `"Revive at safe location: Free"`;
* the buttons emit `fscommand("FR_event","respawn")`, `…"OnPaidRevive( 1 )"` (paid) and
  `…"OnPaidRevive( 0 )"` (safe);
* the countdown path uses the strings `S##RecoverNow`, `S##RecoverIn` and `S##YouGotKnockedOut`,
  which the locale resolves to 408279 "Recover Now!", 408280 "Recover in" and 410188 "You Got
  Knocked Out!".

Wire. The original server's `LibSOE/SOE` enums give the recovered ability and combat opcodes:
`AbilityPacketFailed` 1, `AbilityPacketStartCasting` 3, `AbilityPacketClientRequestStartAbility` 10,
`AbilityPacketAbilityDefinition` 13, `CombatPacketSingleAttackTarget` 3 and
`CombatPacketAttackTargetDamage` 4. No packet *body* is recovered here.

## Community mapping (never merged into the original section)

The numeric ability ids and every balance figure are **community**, taken from the pinned Sanctuary
combat resources and labelled `community` in the checker output:

| Community value | Source |
|---|---|
| profile 11 is Medic, wield 14, fallback/basic/special 3010/3011/3024, energy max 100, regen 4 | `Sanctuary/src/Resources/CombatJobs.json` |
| ability 3024 `AoeDamageHeal`, 889 damage, 254 healing, 10 unit radius, 100 energy, cast 16162 / hit 16161 | `Sanctuary/src/Resources/CombatAbilities.json` (its comment transcribes a "sheet") |

Those numbers line up with the recovered locale names and the recovered effect ids, which is useful
cross-reference, but the sheet itself is not in the lab and the mapping stays community. The checker
fails if a community number appears inside the `original` section.

## Reconstructed boundary

The module implements the recovered constants, the First Aid ladder and the revive-offer arithmetic
verbatim. The **target matrix is reconstructed** on top of the recovered sentences: heal accepts
self/group/ally and refuses a knocked-out target, damage accepts opponents only, revive accepts a
knocked-out group member or ally and refuses a living target or the caster. Only the energy and
knock-out refusals carry recovered client text; every other denial is returned without a string
because the real one was not recovered. The module deliberately does not restrict damaging an
already knocked-out opponent, and a dedicated test records that as an open point instead of hiding
it.

## Explicit gaps

* damage against an already knocked-out opponent: unrecovered, no restriction added;
* ability range, line of sight and reuse timers: unrecovered, no distance rule implemented;
* heal-over-time, critical healing and all historical balance numbers: unrecovered;
* the wire body of `AbilityPacketFailed` and the other ability packets: unrecovered (only opcodes);
* the caller of the Lua `RespawnWindow:ForceRevive()` is native and is not in the pinned Lua corpus;
* the two-button dialog was verified from the decompiled GFX and Lua, not from a native capture.

## Verification

```
& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_037
  RED  stub module: test result: FAILED. 4 passed; 10 failed; 0 ignored
  GREEN: 15 passed; 0 failed; 0 ignored
& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests/pieces -p "test_piece_037.py" -v
  RED  checker absent: Ran 19 tests, FAILED (failures=12, errors=7)
  GREEN: Ran 19 tests, OK
& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_037.py --output <report>\medic-progression.json
  piece 037 evidence ok: medic wield type 14, first aid ladder [250, 281, 313, 344, 375]
rustfmt --check --edition 2024 <both owned Rust files>
  clean
```

The GFX export was produced once with the pinned tool and pinned JDK:

```
& '..\lab\tools\jdk\jdk-21.0.12.1+1\bin\java.exe' -jar '..\lab\tools\ffdec-26.3.0\ffdec-cli.jar' \
  -export script,image '<report>\revive-window' \
  '..\lab\extracted\client-assets\pack_16\006032.gfx'
```

Raw evidence (`locale-medic-strings.json`, `client-and-wire-pins.txt`, `respawn-window-lua.txt`,
`wndRespawn.xml.pin.txt`, the ffdec export, `medic-progression.json`, `rust-red.txt`,
`rust-green.txt`, `python-red.txt`, `python-green.txt`) is under
`lab/reports/piece-037-20261009T035019Z/`. Source hashes: locale dump `f1f22a30…2844`,
`WieldTypeAnimationMappings.txt` `87eaae12…c1cd`, `CharacterStatDefinitions.txt` `2bbf1355…9662`,
`AbilityEffectTypeSettings.txt` `843e9ecd…3436`, `ActorCompositeEffectDefinitions.xml`
`715b84a9…ec31`, `ScriptsBase.lua` `c7c71c7c…7c97`, `wndRespawn.xml` `a5010bcb…7c0d`,
`BaseAbilityPackets.cs` `b4927e79…dc50`, `CombatJobs.json` `2acec676…6979`,
`CombatAbilities.json` `ecf3d2d6…df6a`.

The workspace-wide `cargo test` currently fails in other workers' targets
(`tests/piece_038.rs` includes a `src/restoration/piece_038.rs` that does not exist yet, and
`src/restoration/piece_030.rs` had a serde lifetime error during this run). That is unrelated to
these paths; `cargo test --test piece_037` is green. `cargo clippy` is not installed for the pinned
toolchain, so only `rustfmt --check` was available.

## Next gates

1. Lead integration: register `pub mod piece_037;` under `src/restoration/` and drop the `#[path]`
   include; keep `health`/`energy` ownership with whichever piece owns character stats.
2. Independent review of the reconstructed target matrix, especially the decision that healing never
   revives and that ability revives need another friendly actor.
3. Native acceptance (lead-owned): knock out in the overworld, confirm the two buttons, the disabled
   paid button below cost and the free safe revive, then confirm the countdown path inside an
   encounter.
4. Cross-piece: 036 (Wizard) and 058 (death and revive) should adopt this revert/refusal vocabulary
   rather than duplicating it.

Cross-links: [[../Design/MMO Worker Contract]], [[../../MMO Work Queue]].
