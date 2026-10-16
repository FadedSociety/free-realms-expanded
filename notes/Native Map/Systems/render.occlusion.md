# System: render.occlusion

2 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b60b0` | `OcclusionManager::ScalarDeletingDestructor` | OcclusionManager | Scalar deleting destructor for OcclusionManager: delegates object cleanup to 010b6ac0, conditionally frees this when flag bit 0 is set, and returns this. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b6010` | `— ⚠reject` | OcclusionManager | Constructor of the occlusion manager: it registers itself as an asset-delivery handler on the way in, so it consumes occluder assets; built by 00946a80. | [[Native Map/Shards/S0222 008b1e00-008b7730|S0222]] |

