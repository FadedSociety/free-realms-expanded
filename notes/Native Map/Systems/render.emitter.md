# System: render.emitter

29 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (20)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007c0690` | `ActorModelMaterialEmitter::CallWith0xa0_0x40` | ActorModelMaterialEmitter | Virtual slot 1 of ActorModelMaterialEmitter that forwards four fields of a sub-object at +0xa0 to 0076f2d0, guarded by 007cfe50; the emitter role is taken from the RTTI class name only. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c06c0` | `ActorModelMaterialEmitter::CallWith0xa0_0x58` | ActorModelMaterialEmitter | Virtual slot 2 of ActorModelMaterialEmitter that forwards one field of its +0xa0 sub-object to 0076f390, guarded by 007cfe50. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c06f0` | `ActorModelMaterialEmitter::~ActorModelMaterialEmitter` | ActorModelMaterialEmitter | Destructor body of ActorModelMaterialEmitter (reached from one MSVC EH unwind funclet and listed by 007763d0 and 007836b0). | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c0910` | `ActorLightEmitter::CompareFloat0xbc` | ActorLightEmitter | Virtual predicate of ActorLightEmitter that tests the +0xbc float of its +0xa0 sub-object against a constant; the game meaning of the field is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c0940` | `ActorLightEmitter::ReleaseA8` | ActorLightEmitter | Release of the object held at +0xa8 by ActorLightEmitter; the flag selects a preliminary call (007d01f0). | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1040` | `ActorCameraEffectEmitter::CompareFloat0xa8` | ActorCameraEffectEmitter | Virtual predicate of ActorCameraEffectEmitter testing the +0xa8 float of its +0xa0 sub-object. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1070` | `ActorCameraEffectEmitter::ReleaseA8` | ActorCameraEffectEmitter | Release of the object held at +0xa8 by ActorCameraEffectEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c13c0` | `ActorCameraEffectEmitter::BuildObject` | ActorCameraEffectEmitter | Creates the camera-effect object stored at +0xa8 of ActorCameraEffectEmitter; the body is partly summarised. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c16d0` | `ActorRumbleEffectEmitter::CompareFloat0xac` | ActorRumbleEffectEmitter | Virtual predicate of ActorRumbleEffectEmitter testing the +0xac float of its +0xa0 sub-object. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1700` | `ActorRumbleEffectEmitter::ReleaseA8` | ActorRumbleEffectEmitter | Release of the object held at +0xa8 by ActorRumbleEffectEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1a50` | `ActorRumbleEffectEmitter::BuildObject` | ActorRumbleEffectEmitter | Creates the rumble-effect object stored at +0xa8 of ActorRumbleEffectEmitter; the body is partly summarised. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1d40` | `ActorMeshEmitter::CompareFloat0xac` | ActorMeshEmitter | Virtual predicate of ActorMeshEmitter testing the +0xac float of its +0xa0 sub-object. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1d70` | `ActorMeshEmitter::ReleaseA8` | ActorMeshEmitter | Release of the object held at +0xa8 by ActorMeshEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1dc0` | `ActorMeshEmitter::CallSlot3IsZero` | ActorMeshEmitter | Virtual predicate of ActorMeshEmitter that negates the byte result of its own slot 3. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2160` | `ActorMeshEmitter::BuildObjectIfAbsent` | ActorMeshEmitter | Creates the mesh-effect object at +0xa8 of ActorMeshEmitter only when none exists; the body is partly summarised. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2470` | `ActorLightsaberTrailEmitter::ReleaseA8` | ActorLightsaberTrailEmitter | Release of the object held at +0xa8 by ActorLightsaberTrailEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c24f0` | `ActorLightsaberTrailEmitter::~ActorLightsaberTrailEmitter` | ActorLightsaberTrailEmitter | Destructor body of ActorLightsaberTrailEmitter, reached from one EH unwind funclet and listed by callers 00776800 and 00783f20. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2b20` | `ActorCompositeEffectEmitter::ReleaseA4` | ActorCompositeEffectEmitter | Release of the object held at +0xa4 by ActorCompositeEffectEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2b60` | `ActorCompositeEffectEmitter::~ActorCompositeEffectEmitter` | ActorCompositeEffectEmitter | Destructor body of ActorCompositeEffectEmitter, reached from one EH unwind funclet and listed by callers 00776900 and 007840d0. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2dc0` | `ActorParticleEmitter::ReleaseA4` | ActorParticleEmitter | Release of the object held at +0xa4 by ActorParticleEmitter. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007c0ce0` | `ActorLightEmitter::BuildLightObject` | ActorLightEmitter | Creates the light object held at +0xa8 of ActorLightEmitter from the +0xa0 sub-object's parameters; the body is summarised, not fully decoded. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2fe0` | `ActorParticleEmitter::BuildObject` | ActorParticleEmitter | Virtual slot 1 of ActorParticleEmitter that builds the particle-effect object; the body is not decoded in full, so the stored field is not identified. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |

## ⚪ unknown (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007c09d0` | `— ⚠reject` | ActorLightEmitter | Destructor body of ActorLightEmitter, reached from one EH unwind funclet and listed by callers 007764b0 and 00783860. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c10b0` | `— ⚠reject` | ActorCameraEffectEmitter | Destructor body of ActorCameraEffectEmitter, reached from one EH unwind funclet and listed by callers 00776590 and 00783a10. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1740` | `— ⚠reject` | ActorRumbleEffectEmitter | Destructor body of ActorRumbleEffectEmitter, reached from one EH unwind funclet and listed by callers 00776670 and 00783bc0. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c1de0` | `— ⚠reject` | ActorMeshEmitter | Destructor body of ActorMeshEmitter, reached from one EH unwind funclet and listed by callers 00776720 and 00783d70. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c27c0` | `— ⚠reject` | ActorLightsaberTrailEmitter | Creates the trail object at +0xa8 of ActorLightsaberTrailEmitter from the sub-object's field; the body is partly summarised. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2ab0` | `— ⚠reject` | ActorCompositeEffectEmitter | Builds the composite-effect object stored at +0xa4 from the +0xa0 sub-object; the body is summarised. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c2ee0` | `— ⚠reject` | ActorParticleEmitter | Destructor body of ActorParticleEmitter, reached from one EH unwind funclet and listed by callers 00777060 and 007834e0. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |

