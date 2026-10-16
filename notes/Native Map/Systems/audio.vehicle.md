# System: audio.vehicle

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b9ae90` | `VehicleRaceProcessor::StopLoopingRaceSounds` | VehicleRaceProcessor | Stops the three continuously looping race sound emitters (per the slot roles established by 00b9f360); reached from the race reset/start 00ba5eb0. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |
| `00b9f360` | `VehicleRaceProcessor::CreateRaceSoundEmitters ⚠downgrade` | VehicleRaceProcessor | Race sound setup run when the vehicle-race controller starts a new session: it fills the 24 {event,emitter} pairs that 00b9a660 releases, 00b9ac50 volume-adjusts and 00b9ae90 stops. | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b9ac50` | `— ⚠reject` | VehicleRaceProcessor | Applies the race sound volume to the VehicleRaceProcessor sound emitters created by 00b9f360; the countdown beep/go emitter slots are deliberately not updated. Called from 00ba6530 (race UI/results path). | [[Native Map/Shards/S0361 00b9a660-00ba0260|S0361]] |

