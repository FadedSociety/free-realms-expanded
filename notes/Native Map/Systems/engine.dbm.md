# System: engine.dbm

9 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `008a9990` | `HashListMap_ColumnNameLookupNode_Construct` | SoeUtil::HashListMap | Initialises the DB column-name lookup used by the database-mapping layer. | [[Native Map/Shards/S0221 008a86b0-008b1d80|S0221]] |
| `013c29c0` | `DBMInterface::~DBMInterface` | DBMInterface | Destructor body of the DBM interface base; reached from EH unwind funclets. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c29e0` | `DBMInterface::ScalarDeletingDtor` | DBMInterface | Scalar deleting destructor of the DBM interface base. | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## 🟢 likely (3)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c30c0` | `DirDBM::WriteBlobToFile` | DirDBM | Persists a DBM byte block to a file named by the object (write, then close, then check the byte count). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c55b0` | `DirDBM::~DirDBM` | DirDBM | Scalar deleting destructor of DirDBM. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |
| `013c55d0` | `DirDBM::ScanDirectory` | DirDBM | Enumerates the ruleset/data files in the DirDBM directory and forwards each extension-stripped file name into the entry vector that the index builder consumes. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

## 🟡 speculative (1)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c29d0` | `DBMInterface::SetWordAt4 ?` | DBMInterface | Virtual slot [10] of DBMInterface: stores the argument word at this+4 and returns (bytes 8b 44 24 04 / 89 41 04 / c2 04 00). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `013c39a0` | `— ⚠reject` | DirDBM | Loads a DBM byte block from the file named by the object; the read half of DirDBM::WriteBlobToFile (013c30c0, slot [7]). | [[Native Map/Shards/S0727 013be8e0-013c3b90|S0727]] |
| `013c50a0` | `— ⚠reject` |  | Maintains a timestamp-keyed index of paths gathered from an externally enumerated entry list (likely a directory-backed data cache); role inferred from callees, not verified. | [[Native Map/Shards/S0728 013c3c10-013c8e90|S0728]] |

