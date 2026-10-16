# System: physics

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ae91d0` | `ResolveConvexHitEndpoint` |  | Pushes a requested endpoint out of repeated convex-hit intersections. | [[Native Map/Shards/S0326 00ae4fa0-00ae9b00|S0326]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00aa9d10` | `AabbStrictOverlap ⚠downgrade` |  | Strict axis-aligned box overlap test (touching boxes fail) used by the engine's query code. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aa9eb0` | `AabbInclusiveContains` |  | Inclusive containment test reporting whether box param_2 lies inside box param_1. | [[Native Map/Shards/S0315 00aa90e0-00aae6b0|S0315]] |
| `00aaed30` | `TestTransformedBoundsOverlap` |  | A shared transformed-bounds test; housing and player-position handlers both call it. | [[Native Map/Shards/S0316 00aae6e0-00ab2c20|S0316]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bf8940` | `— ⚠reject` |  | Thiscall(this, obj). Returns if obj is null or vcall +0x24 of obj is zero. Computes two sweep start/end points from this+0x74 and obj's transform (vcall +0x1c). Uses a local SoeUtil::Array<AppPhysics::ConvexHit,0,1> to … | [[Native Map/Shards/S0380 00bf5ff0-00bf9410|S0380]] |

