# System: audio.sound

14 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (11)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007bd670` | `SoundControl3d::Read4Dwords_0x50` | SoundControl3d | SoundControl3d virtual accessor (vtable slot 3) that reads the 16-byte block at +0x50; the game meaning of the block is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd690` | `SoundControl3d::Write4Dwords_0x50` | SoundControl3d | SoundControl3d virtual mutator (vtable slot 4) that writes the 16-byte block at +0x50; the game meaning of the block is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd6b0` | `SoundControl3d::Read4DwordsPair_0x60_0x70` | SoundControl3d | SoundControl3d virtual accessor (vtable slot 5) reading two 16-byte blocks at +0x60 and +0x70; the game meaning is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd6f0` | `SoundControl3d::Write4DwordsPair_0x60_0x70` | SoundControl3d | SoundControl3d virtual mutator (vtable slot 7) writing two 16-byte blocks at +0x60 and +0x70; the game meaning is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd9b0` | `SoundManager::AllocConstruct0x98` | SoundManager | Factory virtual of SoundManager that builds one 0x98-byte object from two ref-counted arguments; the constructed type is identified by callee 007cf450, not by this function alone. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007be020` | `SoundControl::ReadBlocks_0x70_0x60` | SoundControl | SoundControl virtual accessor (vtable slot 5) reading two 16-byte blocks from the object 00769710 returns; the game meaning is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007be070` | `SoundControl3d::SetCosSinFromAngle` | SoundControl3d | SoundControl3d virtual (vtable slot 6) that stores a cos/sin direction built from an angle and forwards it through slot 7; the game meaning of the angle is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007be0c0` | `SoundControl3d::Diff4Floats_0x50` | SoundControl3d | SoundControl3d virtual (vtable slot 9) returning the component-wise difference between its 16-byte block at +0x50 and a supplied vector; the game meaning is not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bea60` | `SoundControl3d::SumSqDiff4_0x50` | SoundControl3d | SoundControl3d virtual (vtable slot 8) returning a squared Euclidean-style difference between its 16-byte block at +0x50 and a supplied vector (four components); game meaning not established. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bff80` | `SoundManager::CallKeyedGroupSlot0` | SoundManager | Calls the first virtual of each object registered under the key of an argument object in the manager's side table; the registered objects' classes are not identified in this function. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007c00e0` | `SoundManager::CallKeyedGroupSlot1` | SoundManager | Calls the second virtual of each object registered under the key of an argument object in the manager's side table. | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |

## 🟡 speculative (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `007bd8f0` | `Write4Dwords_0x50 ?` |  | Thiscall, one stack argument: stores four dwords from the argument into this+0x50..0x5c; ret 4. Same body as SoundControl3d::Write4Dwords_0x50 (007bd690). | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd910` | `Read4Dwords_0x50 ?` |  | Thiscall, one stack argument: copies four dwords from this+0x50..0x5c into the argument buffer; ret 4. Same body as SoundControl3d::Read4Dwords_0x50 (007bd670). | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |
| `007bd930` | `Write4DwordsSwapped_0x60_0x70 ?` |  | Thiscall, two stack arguments: stores the first argument into this+0x70..0x7c and the second into this+0x60..0x6c; ret 8. Differs from SoundControl3d::Write4DwordsPair_0x60_0x70 (007bd6f0), which maps the first argument… | [[Native Map/Shards/S0178 007bd5b0-007c2fe0|S0178]] |

