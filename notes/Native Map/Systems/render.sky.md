# System: render.sky

9 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00807f70` | `LinearBlendController_TintedCtor` | Sky::LinearBlendController<SkyDefinitio… | Constructs a sky linear-blend controller over three TintedTexture blend slots. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00808340` | `LinearBlendController_SatelliteCtor` | Sky::LinearBlendController<SkyDefinitio… | Constructs a sky linear-blend controller over three SkyDefinition::Satellite sub-objects. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008088e0` | `Sky::LinearBlendController_TintedScalarDeletingDtor` | Sky::LinearBlendController<SkyDefinitio… | Scalar deleting destructor of Sky::LinearBlendController<SkyDefinition::TintedTexture>. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `00808900` | `Sky::LinearBlendController_SatelliteScalarDeletingDtor` | Sky::LinearBlendController<SkyDefinitio… | Scalar deleting destructor of Sky::LinearBlendController<SkyDefinition::Satellite>. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `0080a700` | `LinearBlendController_HorizonCtor` | Sky::LinearBlendController<SkyDefinitio… | Constructs a sky linear-blend controller over three Horizon-blend sub-objects. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00808040` | `LinearBlendController_TintedDtor` | Sky::LinearBlendController<SkyDefinitio… | Destructor body of a sky linear-blend controller holding three TintedTexture sub-objects. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |
| `008083d0` | `LinearBlendController_SatelliteDtor` | Sky::LinearBlendController<SkyDefinitio… | Destructor body of a sky linear-blend controller over three SkyDefinition::Satellite sub-objects. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00807ae0` | `Sky::SubmitEnabledLayers ?` | Sky | Thiscall (this, param_2). Returns if this+0x18 is null or the byte at that object's +0x27dc is zero. For each optional handle at this+0x44, +0x48, +0x4c, +0x54, +0x58 and +0x5c it calls 00758cb0(param_2+8, -1) and then … | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0080a790` | `— ⚠reject` | Sky::LinearBlendController<SkyDefinitio… | Destructor body of a sky linear-blend controller over Horizon-blend sub-objects. | [[Native Map/Shards/S0192 00805a50-0080a790|S0192]] |

