# System: audio.fishing

7 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b65d90` | `FishingProcessor::LoadMusicTracks` | FishingProcessor | Builds the fishing minigame's music and ambience set once when the processor starts; the nine ids map one-to-one onto the nine sound-trigger functions FUN_00b61290 tears down. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |

## 🟢 likely (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b614a0` | `FishingProcessor::AllSoundTracksReady` | FishingProcessor | Readiness predicate used before the fishing music state machine switches tracks; the nine-member chain follows the same slots FUN_00b61290 tears down. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b61580` | `FishingProcessor::SaveAudioState` | FishingProcessor | Saves the current audio/mixer state before the fishing minigame changes it (the same mixer object is driven by FUN_00b615e0 and FUN_00b61640); the caller FUN_00b6bb90 pairs all three. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b61640` | `FishingProcessor::RestoreAudioState` | FishingProcessor | Puts the previously saved audio/mixer state back after the fishing minigame leaves its special state; caller FUN_00b6bb90 pairs it with the save and set variants. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |
| `00b65040` | `FishingProcessor::UpdateTrackPositions` | FishingProcessor | Per-frame update that keeps the fishing sound emitters' positions and parameters in step with the local player and their bobber. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b615e0` | `FishingProcessor::SetAudioStateUnderwater ? ⚠downgrade` | FishingProcessor | Sets a muted/filtered audio state; its caller FUN_00b64360 is the fishing underwater transition (it posts Fishing:StartUnderwater), and the previous values are what FUN_00b61580 saved. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b61290` | `— ⚠reject` | FishingProcessor | Teardown of the nine sound emitters the fishing processor creates for its MG_Fishing music tracks; called from the FishingProcessor destructor FUN_00b69d40, which is what identifies the owning class. | [[Native Map/Shards/S0349 00b5ffe0-00b66a80|S0349]] |

