# System: world.camera

5 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb2bc0` | `LoadRailCameras` |  | Populates the client-side rail-camera table at startup; called from the bootstrap loader 0092b170 together with the other client data-file loaders. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb28c0` | `HashListClearAll_8` |  | Erase-all path of the same table that the RailCameras loader 00bb2bc0 drains inline before refilling; the record has no caller rows, so the camera tie rests on the shared layout with 00bb2b10/00bb2bc0. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb2960` | `ScanEntriesMeasureQuaternionDistance → FindClosestEntryByQuaternionDistance` |  | Distance/selection scan over the rail-camera entries for a given actor id (param_2+0x194) and orientation (param_3); candidates 00ae9d30 and 00aefc90 in the camera code are its only callers. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |
| `00bb2b10` | `HashListInsertTail` |  | Insert path of the RailCameras table: the loader 00bb2bc0 hands it the 12 parsed fields of one line and the line's numeric key. | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00bb2720` | `— ⚠reject` |  | Entry lookup of the table whose insert is 00bb2b10 and whose loader 00bb2bc0 parses RailCameras.txt; those two functions prove the identical layout (bucket at +0x14+(k&0x3f)*4, key +0x38, chain +0x3c), and the two calle… | [[Native Map/Shards/S0366 00bb11b0-00bb7710|S0366]] |

