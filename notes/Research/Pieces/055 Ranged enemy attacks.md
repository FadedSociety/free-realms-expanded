# 055 Ranged enemy attacks

Status: ready for review, 2026-10-09 04:33 UTC. This piece recovers the **cast/projectile packet
vocabulary** that a ranged enemy attack rides, the **client-side projectile model** that decides
travel, the **hit-resolution frames** that carry the outcome, and ships one bounded enemy cast
lifecycle plus a provenance checker. It is evidence plus a checker plus a typed module; it is not a
playable claim and no native gate was run.

Owned paths: `prototype/osfr-tools/src/restoration/piece_055.rs`,
`prototype/osfr-tools/tests/piece_055.rs`, `scripts/pieces/piece_055.py`,
`tests/pieces/test_piece_055.py`, this note and `lab/reports/piece-055-20261009T043358Z/`. No shared
`lib.rs`, gameplay, gateway, Cargo manifest, launcher, content, state or native UI file was touched.

## Recovered original facts

Families and sub-opcodes. The supplied server's own enum files and its packet-code table agree with
the preserved client's constructors name for name:

| Family | Id | Evidence |
| --- | --- | --- |
| `BaseCombatPacket` | 32 | `lab/references/OpenSourceFreeRealms/.../PacketCodes.cs` line 17 |
| `BasePlayerUpdatePacket` | 35 | same file, line 20 |
| `BaseAbilityPacket` | 36 | same file, line 21 |

`lab/decompiled/server/LibSOE/SOE/BaseCombatPackets.cs`: `AutoAttackTarget` 1, `SingleAttackTarget`
3, `AttackTargetDamage` 4, `AttackerMissed` 5, `TargetDodged` 6, `AttackProcessed` 7,
`EnableBossDisplay` 9. `BaseAbilityPackets.cs` carries `Failed` 1, `StartCasting` 3, `LaunchAndLand`
4, `ClientRequestStartAbility` 10, `DetonateProjectile` 14, `CastInterrupt` 18.

The client's constructors carry the same immediates inside
`lab/decompiled/client/FreeRealms/pseudocode.c`:

| Class | Immediate | Function |
| --- | --- | --- |
| `AbilityPacketStartCasting` | 3 | `FUN_00a2ed30` |
| `AbilityPacketDetonateProjectile` | 14 | `FUN_00a2ee30` |
| `PlayerUpdateLaunchProjectilePacket` | 62 | `FUN_008d0b10` |
| `CombatPacketAutoAttackTarget` | 1 | `FUN_00a29c20` |
| `CombatPacketSingleAttackTarget` | 3 | `FUN_00a29cf0` |
| `CombatPacketAttackTargetDamage` | 4 | `FUN_00a29dc0` |
| `CombatPacketAttackAttackerMissed` | 5 | `FUN_00a29e90` |
| `CombatPacketAttackTargetDodged` | 6 | `FUN_00a29f60` |
| `CombatPacketAttackProcessed` | 7 | `FUN_00a2a470` |
| `CombatPacketEnableBossDisplay` | 9 | `FUN_00a2a030` |

Partial payload shapes. `PlayerUpdateLaunchProjectilePacket` embeds the client's
`ProjectileParameters`; its constructor `FUN_008d00e0` writes dword `+0x20` and `+0x24` from
`_DAT_0175b420`, which is `0x3f800000` (1.0f) in `.rdata` at VA `0x175b420`, and initialises the
rest from runtime globals (the `.data` globals behind `0x1cb53d0…`, `0x1be6870…` are past the raw
section data, i.e. zero-initialised at runtime and not file-recoverable). The same initialiser also
sets two inline strings at `+0x84` and `+0x94` and three `float4` blocks at
`+0x50`/`+0x60`/`+0x70`; whether every one of those belongs to the embedded struct or to the packet
that owns it was not determined.
`AbilityPacketDetonateProjectile` (`FUN_00a2ee30`) initialises a 64-bit id pair at `+0x10`/`+0x14`
then three zeroed `int32`s at `+0x18`/`+0x1c`/`+0x20`. Field **meanings** are not recovered; only
the constructor shapes are.

Client projectile model. The preserved client names `Projectile`, `ProjectileBeam`,
`ProjectileArc`, `ProjectileThrowCatch`, `ProjectileSeek`, `ProjectileManager`,
`ActorProjectileEmitter`, `ActorProjectileDefinition` (+`ActorProjectileDefinitionManager`),
`PlayerProjectileEvent` and `PlayerSpawnProjectileEvent` in its RTTI. Projectile flight is therefore
a client-side simulation with at least four distinct motion models, bracketed by the server's cast
(`36/3`), launch (`35/62`) and detonation (`36/14`) frames. `AbilityPacketStartCasting` carries the
cast window as a `float ActionTime` plus a progress-bar byte, which is the recovered timing
mechanism; the numeric value per ability is server data and was not recovered.

Hit outcomes and cast refusal. `AttackTargetDamage` 4, `AttackAttackerMissed` 5 and
`AttackTargetDodged` 6 are the source-defined alternatives to a landed hit. `AbilityPacketFailed`
(`36/1`) carries one string id; the client's own `CodeStringMappings.txt` vocabulary has
`NoCastTargeting` 288 and no out-of-range or line-of-sight message, so an over-reach cast is refused
as `NoCastTargeting` - the same convention item 035 uses.

Capture calibration. The preserved ready-stream capture
(`lab/captures/legacy-gateway-20261008T014620082859Z/messages`) frames every message with a 7-byte
gateway header (`u16` channel, `u8`, `u32` inner length). Its 46 messages are 23 x `35/2` (AddNpc),
1 x `36/5` (the ability toolbar), 2 x `38/16`, and single frames of other families - **no `35/62`,
`36/14`, `36/4`, `36/3` or `36/18`**. The private capture therefore cannot validate a projectile
payload.

## Community and other evidence

`lab/references/Sanctuary` (pinned `4f028dcc…`, AGPL-3.0) and the older `FreeRealms-Legacy` copy
both name `35/62`, `36/4`, `36/14` and `36/18` in their reader tables, but neither ships a
serializer for them; the only ability-family serializer is `Ability.cs` (the `36/5` toolbar slot,
including its `float Range`). So the community corpus corroborates the vocabulary and adds nothing
about the payloads.

The preserved asset manifest (`lab/references/osfr-asset-manifest/Assets_manifest.txt`, 162652
rows) holds 180 projectile-named rows: 47 `.adr`, 48 `.xml`, 42 `.dma`, 21 `.dme`, 16 `.dds`, 3
`.gr2`, 2 `.cdt`, 1 `.dsk`. `arrow_projectile.adr` is hash `1685399903`, 132 bytes, with
`.dds`/`.dma`/`.dme` companions; `bolt_projectile.adr` is hash `291596000`, 51 bytes;
`bone_projectile.cdt` and `mail_projectile.cdt` are both hash `2440175293`, 2557 bytes. The
pinned `lab/references/fr-adr-toolkit` can already parse `.adr`, so an extracted projectile actor is
the shortest path to original speeds or arcs - it has not been extracted here.

The read-only wiki corpus (`lab/references/freerealms-wiki/wiki.sqlite`) has no enemy ranged-attack
documentation: `projectile`, `ranged attack`, `spits` and `throws rocks` return zero chunks, and the
single `ranged` hit is the Wizard job description. That is a gap in the corpus, not evidence that
retail enemies never cast.

## Reconstructed policy (ours, not historical)

`piece_055.rs` implements [`RangedEnemyAttack`] - one caster, one target, one caller-supplied
[`EnemyCastProfile`] (`ability_id`, `cast_time_ms`, `flight_time_ms`, `range_units`):

1. `begin_cast(now, distance)` admits the cast only when the distance is inside `range_units`, the
   caster is idle and the profile is sane; otherwise `OutOfRange`, `Busy` or `Invalid`, and the cast
   never starts. The refusal the caller sends is `NoCastTargeting` 288.
2. `begin_cast` returns the `StartCasting` intent (`36/3`; the frame bytes are item 035's encoder)
   carrying `action_time_ms` and the progress flag. The cast completes at `now + cast_time_ms`,
   where `advance` emits either `LaunchProjectile` (`35/62`) when `flight_time_ms > 0` or
   `DetonateProjectile` (`36/14`) when it is zero.
3. A launched projectile detonates at `launch + flight_time_ms`; `resolve` then maps the caller's
   outcome onto `32/4`, `32/5` or `32/6` exactly once.
4. `interrupt()` is only legal while casting and emits `36/18`; in-flight and resolved attacks have
   no recovered cancellation.

Every timing and range number is an input. The module defaults none of them, and a zero flight time
deliberately does not invent a `36/4` frame whose role is unrecovered. Intents carry no payload
bytes: [`EnemyAttackIntent::frame`] returns only the recovered `(family, sub-opcode)` pair.

## Checks the checker enforces

`scripts/pieces/piece_055.py` fails closed (exit 2, named check; exit 0 and `"problems": []`
observed) unless every one of these holds:

* the ten constructor immediates and the three family ids match the tables above;
* the client still names all eleven projectile classes and still builds `ProjectileParameters` from
  `_DAT_0175b420` at dwords 8 and 9, and `AbilityPacketDetonateProjectile` still zeroes dwords 6-8;
* the preserved `FreeRealms.exe` still hashes `26cc1c52…` and the dword at VA `0x175b420` is still
  `1.0f` inside `.rdata`;
* both decompiled server enums still carry the modelled ids;
* both pinned community readers still name `36/4`, `36/14`, `36/18` and `35/62`;
* `NoCastTargeting` is still 288 and no `NoCastOutOfRange`/`NoCastLineOfSight`/`NoCastTooFar` has
  appeared;
* the asset manifest still carries 47 projectile `.adr` rows, exactly 2 `.cdt` rows and the three
  pinned rows with their exact hash/size fields;
* the preserved capture still parses under the 7-byte gateway framing, still holds its 23 `35/2`
  and one `36/5` frames, and still holds **no** cast/projectile frame - the presence of one is
  reported as a problem because it would invalidate this item's central limitation.

The checker also records file sizes and SHA-256 hashes for every input it reads
(`provenance.json`).

## Commands and results

```
.\scripts\cargo.ps1 test --manifest-path prototype\osfr-tools\Cargo.toml --test piece_055
test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

.\scripts\cargo.ps1 build --manifest-path prototype\osfr-tools\Cargo.toml
Finished `dev` profile [unoptimized + debuginfo] target(s)

& '..\lab\tools\wiki-env\Scripts\python.exe' -m unittest discover -s tests\pieces `
    -p "test_piece_055.py" -v
Ran 6 tests in 3.713s - OK (5 hermetic, 1 preserved-corpus)

& '..\lab\tools\wiki-env\Scripts\python.exe' scripts\pieces\piece_055.py --lab ..\lab `
    --out ..\lab\reports\piece-055-20261009T043358Z --stamp 20261009T043358Z
problems [] , exit 0 , evidence_sha256 1da9622a34eb46db52d6307158e9351b456423b56bb3040eb3e21afac3160e0a
```

RED before GREEN, both retained in the report directory: `red-cargo-test.txt`
(`error: couldn't find file tests\..\src\restoration\piece_055.rs`) and `red-python-test.txt`
(`FileNotFoundError: ... scripts\pieces\piece_055.py`, 6 errors).

Crate-wide `cargo test` is currently red on **other workers' in-flight targets** - for example
`tests/piece_044.rs` and `tests/piece_063.rs` reference module files that do not exist yet, and
`tests/piece_083.rs` / `tests/piece_099.rs` fail to compile (`no field changed on type LogRecord`,
`cannot borrow gateway as mutable`, missing `PartialEq` for `ShopCatalogue`). Those files are not
this item's paths; the full output is retained at `crate-wide-cargo-test.txt` for the lead. The
crate's own `cargo build` is green.

## Original vs reconstruction boundary

Original: family ids, all sub-opcode ids and names, the ten constructor immediates, the two partial
payload shapes, the 1.0f `ProjectileParameters` default, the client's projectile class vocabulary,
the combat result vocabulary, the `NoCast*` string ids, the asset-manifest projectile rows and the
capture's frame histogram.

Reconstructed (ours): the cast/launch/detonate/resolve lifecycle, the zero-flight branch, the
single-resolution rule and the `interrupt` restriction. Inferred, not original: that `35/62`
brackets a travelling projectile and `36/14` its detonation - that reading follows the class names
and the `PlayerUpdate`/`Ability` split, not a recovered handler.

## Gaps and next gates

* `35/62`, `36/14`, `36/4` and `36/18` payload layouts are unrecovered; the module returns intent
  types and never fabricates frames.
* No original cast time, projectile speed or enemy reach. The shortest closure is extracting
  `arrow_projectile.adr` (hash `1685399903`) with the existing pack reader and parsing it with the
  pinned `fr-adr-toolkit`, then reading the `ActorProjectileDefinition` fields the client's
  `ProjectileParameters` copies.
* `36/4` `LaunchAndLand` remains a name without a role.
* No native/integration gate: the lead must wire the intent sequence into the encounter and observe
  the original client's cast bar, projectile and hit frame before any playable claim. No damage,
  reward, XP or item path is added here.

## Skills applied

Ponytail (reuse the supplied server enums, the pinned readers and item 035's `36/3` encoder rather
than writing new serializers; the module is one small state machine with one owned test target).
Caveman (concise status). RTK (normal commands first; no blanket prefixing). game-recon (the client
executable, RTTI and packet families, rather than assuming a format). reverse-engineering (Ghidra
pseudocode constructor walks and PE `.rdata` reads). game-research-websearch was not needed: the
pinned community readers and the local wiki corpus already answered the community question, and no
new web source was consulted.
