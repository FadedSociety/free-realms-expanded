# System: game.time

8 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (7)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008eab60` | `GameTimeTimelineCtor` | GameTimeTimeline | Constructs game-time timeline and event list. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008eabe0` | `GameTimeGameEventSequencerCtor` | GameTimeGameEventSequencer | Creates sequencer with owned timeline. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008eacf0` | `GameTimeGameEventSequencerDtor` | GameTimeGameEventSequencer | Cleans sequencer-owned timeline state. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008ead40` | `GameTimeGameEventSequencer::ScalarDeletingDtor` | GameTimeGameEventSequencer | Implements scalar deleting destructor. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008eada0` | `GameEventManagerCtor` | GameEventManager | Creates manager and owned helpers. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008eae50` | `GameEventManagerDtor` | GameEventManager | Releases manager-owned helpers. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |
| `008eae90` | `GameEventManager::ScalarDeletingDtor` | GameEventManager | Implements scalar deleting destructor. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008eac70` | `GameTimeGameEventSequencer::UpdateTimelineFromCurrentTime` | GameTimeGameEventSequencer | Updates the associated timeline from the optional current-time source using its period. | [[Native Map/Shards/S0232 008e82e0-008ec1d0|S0232]] |

