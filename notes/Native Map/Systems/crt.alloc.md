# System: crt.alloc

9 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00c00660` | `FreeListNodeWithAlignment4` | SoeUtil::List<CoinStoreItemRecord,-1> | Releases memory for list nodes using alignment four. | [[Native Map/Shards/S0382 00bfcfe0-00c056a0|S0382]] |

## 🟢 likely (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b537f0` | `FreeAligned4 → SoeUtil::List<PetMoodListPacket::PetMood,-1>::FreeAligned4Callba…` |  | Deallocation hook compiled for a container type; the exe RTTI records this address in slot 3 of the SoeUtil::List<PetMoodListPacket::PetMood,-1> vtable. | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |
| `00b538b0` | `FreeAligned4` |  | Deallocation hook of the SoeUtil::Array<PetName,0,1> instantiation (exe RTTI slot 2 of its vtable). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |

## 🟡 speculative (5)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `0099e900` | `FreeTreeRecursive ?` |  | Generic recursive tree deallocator. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099eb00` | `FreeTreeRecursive ?` |  | Generic recursive tree deallocator variant. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099eba0` | `FreeTreeRecursive ?` |  | Generic recursive tree deallocator variant. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `0099ee50` | `FreeTreeRecursive ?` |  | Generic recursive tree deallocator variant. | [[Native Map/Shards/S0267 0099ca10-009a1c60|S0267]] |
| `00cf8560` | `FreeMemory ?` |  | Frees memory through the configured memory manager. | [[Native Map/Shards/S0425 00cf69c0-00cfc480|S0425]] |

## ⚪ unknown (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b537b0` | `— ⚠reject` |  | Deallocation hook compiled for a container that stores 28-byte nodes; the exe RTTI records this address in slot 2 of the SoeUtil::List<PetMoodListPacket::PetMood,-1> vtable (the list's node-free operation). | [[Native Map/Shards/S0346 00b50330-00b55fc0|S0346]] |

