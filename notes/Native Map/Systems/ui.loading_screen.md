# System: ui.loading_screen

4 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cab620` | `SetLoadingBarProgress` |  | Updates the loading-screen progress bar. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cabab0` | `RenderLoadingScreenFrame` |  | Draws one loading-screen frame while loading continues. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |
| `00cabcb0` | `LoadingScreenThread::Run` | LoadingScreenThread | Runs the loading screen's message and render loop on its worker thread. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

## 🟢 likely (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00cab950` | `UpdateLoadingProgress` |  | Aggregates loader work into loading-screen progress. | [[Native Map/Shards/S0411 00cab620-00cb0bb0|S0411]] |

