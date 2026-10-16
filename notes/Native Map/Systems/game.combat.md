# System: game.combat

11 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008b81e0` | `PlayerHitpointDeltaEvent::PlayerHitpointDeltaEvent` | PlayerHitpointDeltaEvent | Initializes a player hitpoint-delta event. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008b8280` | `PlayerHitpointDeltaEvent::ScalarDeletingDtor` | PlayerHitpointDeltaEvent | Releases a PlayerHitpointDeltaEvent allocation. | [[Native Map/Shards/S0223 008b7790-008bcf40|S0223]] |
| `008eb0a0` | `ProjectileParametersCopyCtor` | ProjectileParameters | Copies projectile parameter state including targets. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008c6470` | `CombatTarget_EngageStep` |  | State-gated target engagement that dispatches the accepted auto-attack or single-attack client routines by distance. Return-code meaning, target sentinel meaning, and approach-helper role remain unresolved. | [[Native Map/Shards/S0225 008c52a0-008ca980|S0225]] |
| `00a29b40` | `CombatProcessor::~CombatProcessor` | CombatProcessor | Destructor/teardown emission for CombatProcessor: the only caller 008a60a0 is a scalar deleting destructor, which invokes this teardown and then frees the object; the field initialisation is the separate function 00a2a1… | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a120` | `CombatProcessor::CombatProcessor` | CombatProcessor | Field-initialising constructor of the client-side combat controller; the two globals it copies are the timestamp sentinel used by 00a2a290. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a290` | `IsStoredTimeAfterNow` |  | Deadline predicate paired with the stamp writer 00a2a6c0, which stores the current value plus a delta; both are reached from the timing caller 0093af90 (00a2a2e0 is the sibling helper). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2a6c0` | `StoreTimestampFromSource` |  | Stamps the combat/ability timer field that 00a2a290 later compares against the current time. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2e7d0` | `CopyIdPairOrDefault` |  | Id-pair getter with a default: the object at +0x78 (set by the state functions in this range) is the only source of a real id, and the callers 008bab70/00973200 are outside this shard. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00a2a160` | `— ⚠reject` |  | Turns the attacker towards its target immediately before the client builds an attack packet (called by 00a2c7d0 and 00a2c880). | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |
| `00a2ac20` | `— ⚠reject` |  | The Tab-target selector reached from the native input dispatcher 00b02880 for action 39 (TabTargetSelect); the vault notes 053 and Native Combat Controls document this behaviour. | [[Native Map/Shards/S0292 00a29620-00a2fe20|S0292]] |

