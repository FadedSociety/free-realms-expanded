# System: asset.mannequin

12 mapped functions. Grouped by confidence; server behaviour below is inference from client code only.

## ✅ confirmed (4)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b87d70` | `MannequinManager::MannequinManager` | MannequinManager | Creates the manager that loads and indexes Mannequins.txt (see 00b885c0 and the parser 00b882f0). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b881c0` | `MannequinManager::~MannequinManager` | MannequinManager | Tear down the mannequin manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b885a0` | `MannequinManager::ScalarDeletingDtor` | MannequinManager | Deleting destructor for the mannequin manager. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b885c0` | `LoadMannequinsTextFile` |  | Entry point that loads the mannequin definition file for the manager; called from the client services initializer 009162f0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## 🟢 likely (6)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b87e00` | `MannequinManager::FindEntry` | MannequinManager | Resolve a mannequin entry by id; used by the proxied-actor predicate at 00b52040 and by 00b56dd0. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b87e30` | `MannequinManager::RemoveEntry` | MannequinManager | Remove a mannequin record when its entry object is destroyed (called by the destroy path 00b88140). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b87fd0` | `MannequinEntry_Init` |  | Constructs one Mannequins.txt record before the parser 00b882f0 fills its fields and inserts it into the manager hash table. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88050` | `MannequinEntry_Destroy` |  | Release the two strings of a mannequin record before the record memory is freed (see 00b88100/00b88140). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88100` | `MannequinEntry_ScalarDeletingDtor` |  | Deleting destructor for a mannequin record (same object size class as 00b87fd0's construction). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b88220` | `MannequinsTextRead` |  | Fetch the text that the caller (00b885c0 with the 'Mannequins.txt' path) passes on to the mannequin parser. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

## ⚪ unknown (2)

| addr | name | class | purpose / does | shard |
|---|---|---|---|---|
| `00b88140` | `— ⚠reject` | MannequinManager | Destroy the entry referenced by the manager field +4 (called by the manager destructor 00b881c0 and by the parser 00b882f0 during error resets). | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |
| `00b882f0` | `— ⚠reject` | MannequinManager | Load the mannequin definition list (Mannequins.txt) into the manager's hash table. | [[Native Map/Shards/S0357 00b84e90-00b8a5f0|S0357]] |

