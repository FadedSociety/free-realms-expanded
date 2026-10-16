# System: render.effect

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a2f7f0` | `DrawAoeRingIndicator ⚠downgrade` |  | Ground AoE ring indicator drawn under the ability targeting state (the ring texture name and the draw call are the anchors). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00cc75e0` | `ClearTrackedEffects` |  | Releases active effects tracked by the gesture component. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |
| `00cc7670` | `UpdateNearbyGesturePointEffect` |  | Updates a rendered effect as gesture sample points become reachable. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cc7230` | `— ⚠reject` |  | Gesture rendering uses this helper to create effects at transformed sample locations. | [[Native Map/Shards/S0416 00cc6c40-00ccbaa0|S0416]] |

