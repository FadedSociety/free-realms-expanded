# System: unknown.guid_lookup

1 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `01082360` | `— ⚠reject` |  | Fastcall: calls 01045d50 and 01045c80 to get a table, then looks up the node whose key dword at +0x4 equals the key at this+0x30 in the 10-bucket table at table+0x10 (bucket = key % 10, chain next at +0x0); returns 1 wh… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

