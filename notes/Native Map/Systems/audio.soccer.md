# System: audio.soccer

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b77d70` | `LoadSoccerSounds` |  | Loads the soccer match's five cue sounds (crowd goal, opposing goal, long whistle, 30-seconds-left, pickup spawn); called from the soccer activity entry FUN_00b7e2d0 and released in bulk by FUN_00b73b20. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b77e30` | `NewAudioBoundObject68 ? ⚠rename` |  | Creates an audio-bound object for the soccer object's +4 field; no caller is recorded. | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b75460` | `— ⚠reject` |  | Creates the runtime handle for a sound definition and stores it in the slot's +0x14 field; its only caller FUN_00b77d70 passes the five MG_Soccer_* sound definitions (crowd goal, whistle, pickup spawn), so the slot +0x6… | [[Native Map/Shards/S0353 00b71490-00b77eb0|S0353]] |

