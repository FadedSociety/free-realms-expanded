# 054 Melee attacks

Status: ready for review, 2026-10-09 04:50 UTC. This piece preserves the reviewed 3 m melee rule,
recovers the original attack animation/timing evidence from the preserved client `Resources/` and
ships a typed melee gate that rejects far hits. It is evidence plus a checker plus a small Rust
module; no native run was made by this worker and it is not a playable claim.

Owned paths: `prototype/osfr-tools/src/restoration/piece_054.rs`,
`prototype/osfr-tools/tests/piece_054.rs`, `scripts/pieces/piece_054.py`,
`tests/pieces/test_piece_054.py`, this note and `lab/reports/piece-054-20261009T043758Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Animation identity, `Resources/CodeAnimationMappings.txt` (preserved original client data):

| enum | data | name |
| --- | --- | --- |
| 33 | 1000 | `com_h2h_attack` |
| 26 | 1001 | `com_h2h_attack_01` |
| 121 | 1099 | `com_swing` |

Wield resolution, `Resources/WieldTypeAnimationMappings.txt`
(`WIELD_TYPE^DESCRIPTION^MELEE_ANIMATION_GROUP^IDLE_ANIMATION_GROUP`): hand to hand (1) plays 1000,
one-hand 1020, two-hand 1040, dual wield 1060, bow 1102, wand 1133, polearm 1080, thrown 0, and the
job wield types 9-19, 22-24, 29, 30 all play **1099** (`com_swing`). The row for wield type 13 is
literally named `Brawler`.

Group and slot rows. `Resources/AnimationGroups.xml` defines `com_swing` id 1099 with
`fallback="1000"` and hand-to-hand slots `com_h2h_attack_01..04` (wieldType 1), plus the
`com_h2h_attack` group id 1000. `Resources/AnimationTypes.xml` carries, inside
`AnimationTreeBranch name="Combat"` -> `name="Hand to Hand"`:

```
<AnimationSlot name="com_h2h_attack_01" id="1001" type="2" priority="10" loadType="2"
               freezeMovement="0" doNotInterrupt="1" boneMaskSuffix="upperbody" />
```

Timing contract, `Resources/AnimationTransitionRules.xml`: entering a combat animation is a single
play at normal speed with no delay and a 0.2 s blend in/out
(`Any -> Combat numLoops="1" speed="1.0" delay="0.0" easeIn="0.2" easeOut="0.2"`), and leaving the
combat animation eases back over 0.6 s in / 0.2 s out
(`Combat -> loc_stand`). These are the original client's animation playback times; the clip length
itself is not recovered here (see gaps).

Reviewed native wire, run `client-baseline-20261008T081219Z` replayed by this piece's checker from
the preserved frames only (no game process): 20 client `26/10` selection-103 melee requests, 20
matching server `32/7` processed hits, request-to-hit latency **15-78 ms**, server-observed request
distances **1.9696-2.4338 units**, player swing animation **1099** on all 20 (the recovered Brawler
row for profile 43), creature hits **101** all targeted the player at <= 2.4338 units with animation
**1001** (plus 10 run and 10 stand locomotion messages). This matches the independent review
`notes/Reviews/Native Cave Completion 081219.md` (20 hits, 1.970-2.434 units, 150 ms bound).

Packet shapes reused as evidence, from the same capture and the pinned community schemas:
`26/10` is `i64 guid + i32 selection` (103 = attack, 101 = exit, 104 = guide start) and `32/7` is
`u64 caster + u64 attacker + u64 target + i32 damage + ...`.

## Cross-checks the checker enforces

`scripts/pieces/piece_054.py` fails closed (exit 2, named check) unless all of these hold:

* the three `CodeAnimationMappings.txt` rows above keep their enum/data ids;
* wield type 13 is still the `Brawler` row playing 1099, and hand to hand still plays 1000;
* `com_swing` is still id 1099 with fallback 1000 and the hand-to-hand slot, `com_h2h_attack` is
  still 1000, and the `com_h2h_attack_01` slot row still reads id 1001 / type 2 / priority 10 /
  loadType 2 / freezeMovement 0 / doNotInterrupt 1 / `upperbody` under Combat -> Hand to Hand;
* the `Any -> Combat` and `Combat -> loc_stand` transition timings are unchanged;
* the owned module's constants equal all of the above and its refuse variants are unchanged;
* production `prototype/osfr-tools/src/encounter.rs` still carries the reviewed melee rows
  (`2/11/32/43` and the fallback at reach 3.0), the ranged rows (35 at 30.0, 12 at 20.0), the
  `> range` far-hit rejection, the `600 ms` repeat gate and the `<= 2.5` enemy contact rule;
* the capture replays to exactly 20 requests / 20 hits / 20 swings of 1099 / 101 creature hits of
  1001, every request inside 3.0 units, every latency under 150 ms and the reviewed maximum still
  <= 2.44 units.

## Reconstructed boundary

The reaches and the repeat gate are reconstructed server policy, not recovered client values:
player melee 3.0 units, enemy contact 2.5 units, 600 ms between two accepted player attacks. They
come from `AGENTS.md` ("combat melee remains 3m"), `notes/Design/Opening Playthrough.md` ("melee
limits of three player units and 2.5 enemy units") and the accepted review above. Pursuit speed and
damage numbers stay with the encounter server.

The module is a pure gate: finite positions -> dead-target refusal -> `distance > reach` refusal
(exactly 3.0 is accepted, the next representable float above is refused) -> repeat-gate refusal,
returning the solved distance and the animation the client should play. Damage, rewards, packets and
the action bar remain owned by `encounter.rs` and the gateway.

## Explicit gaps

* The original `com_h2h_attack_01` clip duration and hit frame window are unrecovered: reading the
  Granny `.gr2` animation curves was not performed, so the module records the transition timing
  contract only and never invents a swing duration.
* Original damage, balance and reward amounts are not part of this piece.
* Original action-bar routing is still unproven: the four native `36/10` requests in this run
  produced no processed attack, and all 20 accepted hits came from `26/10` selection 103.
* No native test rejects a hit at exactly the 3.000 m boundary; the capture proves accepted close
  hits (<= 2.4338 units) and the unit tests prove the boundary behaviour.
* `AnimationGroups.xml` is not strict XML (decorative comments contain `--`), so the checker strips
  comments before parsing; that property is preserved, not repaired.

## Verification

```
& ..\..\scripts\cargo.ps1 test --test piece_054
  RED before implementation: error: couldn't find file `tests\..\src\restoration\piece_054.rs`
  GREEN: 8 passed; 0 failed
& '..\lab\tools\wiki-env\Scripts\python.exe' 'tests\pieces\test_piece_054.py' -v
  RED before implementation: RustModuleUnreadable: constant PLAYER_MELEE_REACH missing
  GREEN: 8 passed (Ran 8 tests, OK)
& '..\lab\tools\wiki-env\Scripts\python.exe' 'scripts\pieces\piece_054.py' `
      --output '..\lab\reports\piece-054-20261009T043758Z\piece-054-evidence.json'
  piece 054 evidence ok: 20 melee requests, 20 processed hits, max distance 2.434,
  max latency 78.0 ms, swing [1099]
& ..\..\scripts\cargo.ps1 test --test encounter
  GREEN: 15 passed; 0 failed; 2 ignored (includes
  `melee_requires_contact_and_enemy_motion_attacks_have_native_animation`)
```

The crate itself compiles (`Compiling osfr-tools`, no error in the library) and the production
encounter target still passes, so the reviewed rule is intact in the live server module. A whole-tree
`cargo build --tests` currently stops on five other workers' in-flight targets (`piece_044`,
`piece_047`, `piece_063`, `piece_083`, `piece_099`), which is recorded in the lab log and is not
caused by this piece.

Evidence in `lab/reports/piece-054-20261009T043758Z/`: `piece-054-evidence.json` (provenance hashes
plus the recovered rows and capture replay), `rust-piece-054-tests.log`, `py-piece-054-tests.log`,
`py-piece-054-tests-runtime.log` (the same eight tests under the default runtime interpreter),
`rust-encounter-tests.log`, `rust-build-tests.log`, `piece-054-checker.log`, `red-baseline.log` and
`measure-capture.py` (the scratch measurement that grounded the replay bounds).

## Next gates

1. Lead integration: register `pub mod piece_054;` under `src/restoration/` and drop the `#[path]`
   include from `tests/piece_054.rs`. `encounter.rs` keeps ownership of the encounter itself.
2. Optional server adoption: replace the inline `distance(position, s.position) > range` and
   600 ms checks in `encounter.rs::attack` with this gate so the reviewed rule has one owner; that
   is a lead-owned edit to a shared file.
3. Native acceptance (lead-controlled): stand at ~3.0 m and confirm the client refuses the hit, then
   step to ~2.9 m and confirm one accepted hit plus the 1099 swing; no new run was made here.
4. Evidence follow-up: a Granny GR2 read of `com_h2h_attack_01` would turn the clip duration from a
   gap into a recovered constant.

Cross-links: [[../Native Combat Controls]], [[../Native NPC Motion and Animation]],
[[../../Reviews/Native Cave Completion 081219]], [[../../Design/MMO Worker Contract]],
[[../../MMO Work Queue]].
