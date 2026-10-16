# System: audio.minigame

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad4e80` | `ClientTradingCardGameManager::ApplyMusicLevels` | ClientTradingCardGameManager | Applies the two TCG music channel values to audio controls and persistent settings. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## 🟡 speculative (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad3f30` | `SetMusicLevelB ? ⚠rename` |  | Shared virtual implementation in both card manager classes; writes setting 0xb. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |
| `00ad3f80` | `SetMusicLevelC ? ⚠rename` |  | Shared virtual implementation in both card manager classes; writes setting 0xc. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

