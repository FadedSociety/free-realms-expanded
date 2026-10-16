# System: lib.soegems

13 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b93f70` | `TextIndex::BuildWeightedResults` | SoeGems::TextIndex<int> | Builds scored TextIndex results from query terms and the requested result limit. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |

## 🟢 likely (10)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b93020` | `RebuildDocumentIdLookup` |  | Rebuilds document identifier lookup used by the text-index structures. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93210` | `OwnedStringArrayDeletingDtor` |  | Scalar deleting destructor for an object with owned string and instance-array members. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93230` | `ConstructStringKeyedEntry` |  | Creates one text-index entry keyed by the supplied string. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b932b0` | `AppendHashedOccurrence` |  | Adds an occurrence node to a bucketed text-index structure. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93800` | `DestroyDocumentSubtree` |  | Releases the recursive entry tree used by the adjacent TextIndex structures. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93840` | `InsertTokenOccurrence` |  | Adds a normalized token occurrence; called by the document tokenizer at 00b93d70. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93950` | `ComputeNormalizedCountScore` |  | Returns a score consumed by the weighted-result builder at 00b93f70. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93d00` | `ClearDocumentTree` |  | Clears the document tree used by the surrounding text-index structures. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93d70` | `IndexDocumentTokens` |  | Adds normalized tokens for one document to TextIndex; enclosing manager context is shown by caller 00b95140. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `00b93ef0` | `TextIndexReset` |  | Used by TextIndex construction and refresh paths to clear indexed state. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b93180` | `— ⚠reject` |  | Creates one document lookup entry. | [[Native Map/Shards/S0359 00b90750-00b94a30|S0359]] |
| `010818b0` | `— ⚠reject` |  | One-time creation of the SoeGems singleton object (0x330 bytes) used as the memory pool for RefArray buffers. | [[Native Map/Shards/S0580 0107d0c0-010829c0|S0580]] |

