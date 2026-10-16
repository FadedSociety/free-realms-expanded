# Class: SoeUtil::HashListMap<int,ClientQuestData,256,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c7c6b0` | `ClientQuestDataStore::Destruct` | ✅ | dtor | Drains nodes from the 256,8 map, frees the auxiliary tree at +0x414 recursively, clears tree fields, switches to the 256,-1 map vtable and drains that list too. |
| `00c7c7b0` | `ClientQuestDataStore::DeletingDtor → SoeUtil::HashListMap<int,ClientQuestData,2…` | ✅ | dtor | Calls the quest-data store destructor body and frees the object when flag bit 0 is set; otherwise returns its pointer. |
| `00c7c7d0` | `ClientQuestDataStore::Construct → SoeUtil::HashListMap<int,ClientQuestData,256,…` | ✅ | ctor | Initializes typed hash-map fields and 256 buckets, installs the 256,8 map vtable, and invokes FUN_00baa5c0 for additional initialization. |
