# System: render

6 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c165a0` | `SetFloraEnabled` |  | Toggles terrain flora rendering and persists the matching "Terrain/RenderFlora" option; the name is the table row 0x0182e458 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b564d0` | `DrawPetPathIndicators` |  | Draws the pet follow/path visual indicators; this interpretation follows its pet-handler caller and asset/bone strings. | [[Native Map/Shards/S0347 00b56090-00b5b410|S0347]] |
| `00c164f0` | `GetRenderDistanceInPercent` |  | Reports the render-distance option as a percentage for the options UI; the name is the table row 0x0182e420 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |
| `00c16560` | `SetShadowQuality` |  | Sets the shadow-quality graphics option; the name is the table row 0x0182e438 of the global script table. | [[Native Map/Shards/S0384 00c0dcd0-00c171d0|S0384]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00ad3800` | `CopyTextureRows ?` |  | Iterates texture entries, obtains source and destination surfaces, creates a matching resource when dimensions differ, then copies either the whole image or each row using memcpy and releases the surface. | [[Native Map/Shards/S0323 00ad3270-00ad83b0|S0323]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c1c690` | `— ⚠reject` |  | Reads the RenderFlora setting under Terrain. | [[Native Map/Shards/S0385 00c17390-00c1f0a0|S0385]] |

