# 036 Wizard progression

Status: ready for review, 2026-10-09 03:50 UTC. This piece recovers the Wizard **job kit, spell
definitions, resource pool and effect ids** and ships one validated cast slice that rejects invalid
targets and repeated casts. It is evidence plus a checker plus a small typed module; it is not a
playable claim and no native gate was run.

Owned paths: `prototype/osfr-tools/src/restoration/piece_036.rs`,
`prototype/osfr-tools/tests/piece_036.rs`, `scripts/pieces/piece_036.py`,
`tests/pieces/test_piece_036.py`, this note and `lab/reports/piece-036-20261009T033448Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Ability packets. The decompiled original server enum
`lab/decompiled/server/LibSOE/SOE/BaseAbilityPackets.cs` names 18 ability packets on base opcode
`0x0024` = 36, including `AbilityPacketFailed = 1`, `AbilityPacketStartCasting = 3`,
`AbilityPacketSetDefinition = 5`, `AbilityPacketClientRequestStartAbility = 10` and
`AbilityPacketAbilityDefinition = 13`. Those are the ids this piece models.

Two recovered literals in `Gateway/Gateway.Player/UpdateStats.cs` pin the opening resource and toolbar
behaviour:

* `SendClientUpdatePacketMana` tunnels packet `0x0026` = 38 sub `13` with `100`, `100`, `true`;
* `SendAbilityPacketSetDefinition` sends the literal
  `24000500010000000800000000000000…`, which decodes to opcode 36, sub 5, profile 1, slot count **8**,
  then 32 zero bytes = eight empty `Type 0` slots. The whole literal is 44 bytes.

Source selection rule (also original). Target selection requires hostile `Disposition` 0 at actor
`+0x178`, rejects the dead flag at `+0x6f8` bit 7, and applies a facing dot product plus a line-of-sight
test (`notes/Research/Native Combat Controls.md`). This piece consumes those as call inputs; it does not
reimplement native selection.

Proximity and melee remain the previously reviewed rules (12 m interaction, 3 m melee). The Wizard
reach below is a **job kit** value, not a re-derivation of those.

## Community-recovered data (pinned local references)

`lab/references/Sanctuary` is pinned at revision `4f028dcc274f628b65ef665fd59b184f5f3d3e3e`
(AGPL-3.0, source-only research; no implementation was copied).

| Field | Value | Source |
| --- | --- | --- |
| profile | 12, "Wizard. Ranged; reach 20u…Creation kit = 4914" | `CombatJobs.json` |
| auto-target reach | 20 world units | `CombatJobs.json` |
| wield type | 10 | `CombatJobs.json` |
| slot definitions | basic 4895, special 4899 ("client-known") | `CombatJobs.json` |
| basic recast | 600 ms | `CombatJobs.json` |
| bare-hand fallback | ability 4010 | `CombatJobs.json` |
| energy | max 100, regen 4 per second | `CombatJobs.json` |
| Student Wizard Wand | item 4914 -> Zap 4011 / Lightning Blast 4028 | `CombatJobs.json` |

Ability definitions, `CombatAbilities.json`:

| Id | Name | Effect | Damage | Energy | Animation | Cast FX | Hit FX | Target anim | Icon / NameId |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 4010 | bare-hand fallback | SweepDamage | 200 | 0 | 1099 | - | 7 | 1120 | 280 / - |
| 4011 | Zap | SweepDamage | 279 | 0 | 1099 | - | 7 | 1120 | 14434 / 17071 |
| 4028 | Lightning Blast | SweepDamage | 1143 | 100 | 1138 | 16305 | - | 1120 | 294 / 442459 |

Item 4914, `ClientItemDefinitions.json`: "Student Wizard Wand", `Class` 10 (equal to the kit's wield
type), slot 7, NameId 11056, DescriptionId 1114, icon 3158 tint 234, model
`wand_ar_ag_weapon_crude.adr`, category 207, power rating 1, `NoTrade`, min profile rank 1.

Profile 12, `Profiles.json`: Type 2, NameId 2280, DescriptionId 2281, icon 1343, badge image set 12,
`MembersOnly` false, `Trial` true.

Wire shapes, community packet sources: `AbilityPacketStartCasting` (36/3) carries caster guid, target
guid, composite effect, animation, ability id, `ActionTime` and a progress-bar flag.
`AbilityPacketClientRequestStartAbility` (36/10) carries an `ActionBarData` pair (action bar id, slot),
an `int32` target flag, then a `Vector4` position when the flag is 1 or a `ulong` guid otherwise — 32 or
24 bytes total. `AbilityPacketFailed` (36/1) carries one `StringId`.

Wiki corpus (read-only, `lab/references/freerealms-wiki/wiki.sqlite`):

* Student Wizard Wand (revision 26356) states Zap deals **279** and Lightning Blast **1143** — an
  independent corroboration of the two community damage values;
* Wizard (revision 46067) lists Zap / Fireball / Ice Nova / Chaos at job levels 1 / 5 / 10 / 15, and
  the separate Wizard Power Attacks page lists ~20 weapon power attacks (Arcane Chain, Chain Lightning,
  Firestorm, Ice Nova, Lightning Blast, Tsunami, …). The level list and the weapon list are **different
  axes**: the recovered kit only binds the Student Wand's two abilities, so nothing here invents a
  level-to-ability table.

## Cross-checks the checker enforces

`scripts/pieces/piece_036.py` fails closed (exit 2, named check) unless every one of these holds:

* the kit values above, the wand mapping and the energy pair match `CombatJobs.json`;
* items 4010/4011/4028 match `CombatAbilities.json` field by field (damage, cost, animation, cast/hit
  effect, target animation, name/icon ids);
* item 4914 matches `ClientItemDefinitions.json` and its `Class` equals the kit wield type;
* profile 12 is the Wizard job profile in `Profiles.json`;
* `BaseAbilityPackets.cs` still carries the five modelled ids;
* `UpdateStats.cs` still carries the 38/13 mana literal and the 44-byte 36/5 toolbar literal, and the
  decoded slot bytes are all zero;
* the community packet sources still expose the recovered cast fields and request shape;
* the wiki revisions still carry `279` / `1143` and the four level markers.

It also records two unresolved conflicts instead of smoothing them: the wiki calls Wizard a
"Members Only" job while `Profiles.json` has `MembersOnly` false / `Trial` true, and the wiki level
list does not correspond to the weapon-bound abilities the kit actually maps.

## Reconstructed boundary

Nothing numeric was invented. The module implements the recovered kit numbers, the recovered ability
table and the recovered effect ids, plus an explicitly local validation policy:

1. active profile must be Wizard and the toolbar must agree;
2. the requested slot must resolve to a recovered Wizard ability;
3. the caster position must be finite;
4. a guid target must resolve to the supplied candidate, be finite, alive and hostile
   (`Disposition` 0), and lie inside the recovered 20 unit reach (20.0 included, 20.1 rejected);
5. a `target == 1` request is only checked for finite coordinates — positional resolution stays
   unrecovered;
6. one in-flight cast per caster for `ActionTime`, then the recovered 600 ms basic recast window;
7. the recovered energy cost is spent once on acceptance and the pool regenerates 4 per whole second,
   carrying partial seconds and saturating at 100.

The cast result returns the recovered animation, cast/hit effect and target animation ids together with
the damage and the energy after; damage application, hit scheduling and rewards stay with the owning
subsystems.

## Explicit gaps

* `ActionTime` is unrecovered for every ability, so it is a caller input, not a constant.
* No denial string id is recovered; `CastDenial` deliberately carries no `AbilityPacketFailed` payload.
* Hit timing, projectile travel, the `SweepDamage` cone/area shape and line of sight are unrecovered.
* Whether the server re-validates `AutoTargetReach` for specials, and any per-ability reach, is
  unrecovered; this piece applies the single recovered job reach to both slots.
* Job experience, level unlocks and quest rewards are outside this slice.
* Ability icons/names are carried as ids only; their locale strings are not resolved here.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_036
  RED  before implementation: error: couldn't find file `tests\..\src\restoration\piece_036.rs`
  GREEN: 16 passed; 0 failed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_036.py' -v
  GREEN: 12 passed (Ran 12 tests, OK)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_036.py' `
      --output '..\lab\reports\piece-036-20261009T033448Z\wizard-evidence.json'
  piece 036 evidence ok: wizard reach 20.0, energy 100/4 per second, zap 279, blast 1143
```

Evidence in `lab/reports/piece-036-20261009T033448Z/`: `rust-red.txt`, `rust-green.txt`,
`python-green.txt`, `wizard-evidence.json` and `source-provenance.txt` (sha256 of every input, the
Sanctuary revision and the `combat.rs` cross-check lines). The 24/32 byte request lengths asserted here
match the existing sandbox parse at `prototype/osfr-tools/src/combat.rs:205-213` (`(36, 10)`,
`if target == 1 { 32 } else { 24 }`), which is read-only for this item.

## Next gates

1. Lead integration: register `pub mod piece_036;` under `src/restoration/` and drop the `#[path]`
   include; keep `combat.rs` as the owner of the encounter and the (36/10) parse.
2. Wire integration: send the recovered 36/5 toolbar with slots 4895/4899 and the 36/13 definition
   packets, the 38/13 mana update, then 36/3 start casting and 32/7 attack processed for the resolved
   hit. Do not fabricate `ActionTime`; take it from a capture or keep it a deliberate local value.
3. Native acceptance (lead-controlled): switch to Wizard, equip the Student Wizard Wand, cast Zap and
   Lightning Blast, confirm the animation/effect ids and the energy bar, and verify a repeated request
   inside the lock/recast window is refused instead of double-spending or double-damaging.
4. Job progression (items 032-038): level/experience/unlock tables still need a source; this piece
   intentionally contains none.

Cross-links: [[../Native Combat Controls]], [[../../Design/MMO Worker Contract]],
[[../../MMO Work Queue]].
