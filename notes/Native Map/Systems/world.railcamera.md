# System: world.railcamera

3 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0107fb50` | `ZeroInitRecord0x30 ? → RailCameraRecordInitDefaults` |  | Zero-initialises a record: clears the dwords at +0x0,+0x4,+0x8,+0xc,+0x14,+0x18,+0x1c,+0x28,+0x2c, the bytes at +0x10 and +0x20, and stores the default dword _DAT_0175b420 at +0x24. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008ee710` | `— ⚠reject` |  | Sets the rail-camera control option and applies the matching camera state when the controller is in a compatible state. | [[Native Map/Shards/S0233 008ec2a0-008f0cc0|S0233]] |
| `0107fb90` | `— ⚠reject` |  | Thiscall parser: consumes caret-delimited (0x5e) tokens from the row at param_2 into the record fields: integers via 0072d900 at +0x0, +0x4, +0x8, +0xc, +0x18 and +0x28; values via 0072d550 at +0x10 and +0x20; floats vi… | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

