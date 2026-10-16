# 035 Archer progression

Queue item 035. Acceptance: recover ranged actions/ammo/projectile behaviour; range and
line-of-sight evidence.

Status 2026-10-09 03:40 UTC: the ranged-action wire (client request, cast, toolbar, failure), the
recovered failure vocabulary, the community Archer kit and its reach evidence are recovered and
tested. The three projectile *payload* layouts, a numeric original weapon reach, an ammo economy
and a retail line-of-sight predicate are **not** recovered and are recorded as explicit gaps. No
native or gameplay acceptance was attempted and the module is not registered in `lib.rs`.

## Artifacts

| Path | What |
| --- | --- |
| `prototype/osfr-tools/src/restoration/piece_035.rs` | Recovered 36/1, 36/3, 36/5 and 36/10 codecs, the Archer kit constants, and the reconstructed reach/visibility/ammo policy (new) |
| `prototype/osfr-tools/tests/piece_035.rs` | 15 integration vectors; includes the module by `#[path]` until the lead registers it (new) |
| `scripts/pieces/piece_035.py` | Read-only checker over the preserved client, two pinned community readers, the older decompiled community enum, the client locale table and the original capture corpus (new) |
| `tests/pieces/test_piece_035.py` | 14 stdlib tests: hermetic parser/bundle cases plus a real-corpus pin (new) |
| `../lab/reports/piece-035-20261009T033649Z/` | RED/GREEN logs, `evidence.json`, `provenance.json`, `run.log`, the private probes |

No shared `lib.rs`, gameplay, gateway, Cargo, launcher, content, save or release path was touched.

## Recovered original facts

1. **The ability family is 36 and the projectile launch rides PlayerUpdate.** The supplied client's
   own constructors carry the sub-opcode immediates: `AbilityPacketStartCasting` is built with `3`
   (`FUN_00a2ed30`), `AbilityPacketDetonateProjectile` with `0xe` (`FUN_00a2ee30`) and
   `PlayerUpdateLaunchProjectilePacket` with `0x3e` = 62 (`FUN_008d0b10`). Three independent
   community copies - the two pinned reader switches and the older decompiled `PacketCodes.cs`
   enum - name the same 17 sub-opcodes, name for name, and the checker asserts all three agree.
2. **The client's ranged request is 36/10.** `AbilityPacketClientRequestStartAbility` is
   `[int16 36][int16 10][int32 ActionBarId][int32 Slot][int32 Target]`, then a `float4` world
   position when `Target == 1` and otherwise a `uint64` guid, and the frame must have no trailing
   bytes. That positional form is the shape a ranged, ground-targeted action uses.
3. **The toolbar (36/5) carries the client's range field.** `[int32 ProfileId][int32 SlotCount]`
   then eight positional slots; a slot is `int32 Type`, then `InstanceId/ManaCost` (types 1 and 3)
   or `ItemDefinitionId` (type 2), then `IconId, NameId, Unknown7, Range(float), Unknown9,
   AbilityDefinitionId, Unknown11` and the trailing `ForceDismount` byte. The client hardcodes
   eight slots, so an empty slot hides every later one. Pinned PR 121 documents `Unknown8` as
   `float Range`, reverse-engineered from the client "which reads the field as a float and uses it
   as the auto-target radius".
4. **The refusal vocabulary is the `NoCast*` family.** `AbilityPacketFailed` (36/1) is one `int32`
   string id, and the client's own `CodeStringMappings.txt` names that id space:
   `NoCastKnockedOut` 280, `NoCastAsleep` 281, `NoCastSilenced` 282, `NoCastAfraid` 283,
   `NoCastStunned` 284, `NoCastInsufficientMana` 285, `NoCastDefault` 286, `NoCastRecastTimer` 287,
   `NoCastTargeting` 288 and `NoCastNotEnabled` 3079. **There is no out-of-range and no
   line-of-sight message in that vocabulary.**
5. **The cast frame is 36/3.** `[uint64 caster][uint64 target][int32 CompositeEffectId]
   [int32 Animation][int32 AbilityId][float ActionTime][bool HasActionProgress]`; the client's
   constructor default `-1` for the animation field lands on exactly that dword, which is the
   independent cross-check the Rust vector pins.
6. **Bows are identified in the client's own item tables.** `ItemClasses.txt` row 18 is the
   `WIELD_TYPE 11` bow class (`PROFILE_NAME_ID` 4822) and `ItemCategories.txt` row 207
   (`NAME_ID` 2412) is the bow category every `*Bow` item sits in.

## Community reconstruction (not original server data)

The pinned community repository's `CombatJobs.json` / `CombatAbilities.json` (PR 121, "Add combat
job toolbars with starter weapon ability mappings"; its body notes that ability execution was
deliberately out of scope) documents the Archer kit: profile 35, wield type 11, **auto-target
reach 30 units**, starter bow item 4266 mapping to basic ability 1010 (Barrage, sweep, 279 damage)
and special ability 1011 (Volley, AoE, 506 damage, radius 10, 100 energy). The same table puts
every melee kit at 7 units and the Wizard at 20, which is what makes the 30-unit kit ranged. These
numbers are community reconstruction; treat them as a claim to verify, not as recovered balance.

The wiki agrees on the shape and disagrees with nothing: the Archer page calls the job
"long-distance combat using bows" with traits at levels 5/10/15/20 (Precision, Marksmanship,
Reflexes, Lucky Shot), and the bow pages list the two bow effects by name (Barrage, Volley).

## Reconstructed behaviour

`ReachGate` is our admission test: squared `float4` distance against the caller-supplied reach,
which mirrors the only recovered range semantics (the pinned community helper). An over-reach
request is refused with `NoCastTargeting` **because retail has no range message**; a caller that
recovers a distinct string id must replace that instead of inventing one.

`admit_ranged_action` is the single entry point the lead should call: it runs the reach test first
and the caller-supplied geometry second, and returns `Ok(())` only for "may proceed to cast". It
grants nothing.

`LineOfSight` defaults to `Unchecked`. It only refuses when the caller supplies AABB blockers, and
then it is a plain straight-line segment test in world `(x, y, z)` - geometry, not simulation. No
gravity, arc, travel time or hit timing is inferred anywhere in this module.

`AmmoPolicy` never charges an item. The corpus holds no consumable ammunition evidence: no `ammo`
page in the wiki, no ammo item in the 58,795-row recovered item export, and the single
`Point Blank Archer Quiver` prize is a clothing item. Charging arrows here would be invented
economy, so the type exists to make that refusal explicit.

## Gaps and what would close them

| Gap | Missing | Would close it |
| --- | --- | --- |
| Projectile payloads | field layout of 36/4 LaunchAndLand, 36/14 DetonateProjectile and 35/62 LaunchProjectile | an original capture of an archer shot, or a client parser walk showing the read order |
| Original archer reach | any numeric reach from the game itself | source or client data with a non-zero `Range` for category 207, or a retail ability capture |
| Ammunition | a consumable arrow/quiver path | a recovered ammo item with a consumption path, or a capture charging inventory per shot |
| Line of sight | the retail predicate | recovered server ability handling, or a capture refusing a shot through geometry |
| Native acceptance | a client run of a ranged action | the lead wiring these frames and observing the cast bar and projectile |

The capture corpus supports the projectile gap: `reports/original-minigame-packets` holds 65,810
frames across families 33, 34, 39, 41, 49, 76, 102, 141, 147 and 167 and **zero** frames in
family 36 or PlayerUpdate 62. The checker fails loudly if that changes.

The line-of-sight gap has three independent negatives: the client's `CodeStringMappings.txt` has no
`sight`, `obstruct` or `NoCast*` range message (the only near-hits are unrelated `CoverOver`,
`RecoverIn`/`RecoverNow`); the 95 MB client pseudocode contains no `sight`/`obstruct` predicate;
and neither pinned community reader ships an ability handler (PR 77's body says "the ability
handler and combat systems come in follow-up PRs"). Refusing a shot through geometry is therefore
our policy, kept opt-in and labelled.

## Checks

```
& .\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_035
  -> 15 passed; 0 failed

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces `
    -p "test_piece_035.py" -v
  -> Ran 14 tests ... OK

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_035.py `
    --lab ..\lab --out ..\lab\reports\piece-035-20261009T033649Z --stamp 20261009T033649Z
  -> problems: []  archer_reach: 30  ranged_capture_frames: {}
     evidence_sha256: 8aa804dc63ffa8796fe9a40cb1e95d3f9de4fbd9e0e319e6464b4f13d5d18834
```

RED evidence is `../lab/reports/piece-035-20261009T033649Z/red-module-missing.log`
(`error: couldn't find file tests\..\src\restoration\piece_035.rs`) and
`red-composed-gate.log` (`unresolved import piece_035::admit_ranged_action`); the green runs are
`green-first-run.log`, `green-second-run.log` and `green-third-run.log`. The whole-package suite
cannot run today because other workers' in-flight targets (`tests/piece_038.rs`,
`tests/piece_042.rs`) reference modules that do not exist yet, so this item ran the library target
plus representative sibling targets (`focused-rust-subset.log`: lib 24, combat 5+1 ignored,
piece_026 12, piece_035 15 - all passing).

## Integration instructions for the lead

When `src/lib.rs` and `src/restoration/mod.rs` are updated, register
`pub mod piece_035;` and drop the `#[path]` include from `tests/piece_035.rs`. The module grants no
currency, items or experience, so it can sit behind the lead-owned economy and gateway layers
unchanged.
