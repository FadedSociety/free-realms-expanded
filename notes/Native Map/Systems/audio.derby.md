# System: audio.derby

1 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `009ec910` | `SoundEvent_CreateInstance` |  | Instantiates one live handle from a named event definition; 009ee280 calls it once per derby sound name (Vehicle_Countdown_Beep ... Vehicle_Ultimate_Expired), so the handles it fills are the sound-event instances the re… | [[Native Map/Shards/S0281 009ea8a0-009f0100|S0281]] |

