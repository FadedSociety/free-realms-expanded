# Class: SoeUtil::HashList<AdventurersJournalRegionDefinition,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a453f0` | `HashList_RegionDefinition_Ctor` | ✅ | ctor | Constructor of the derived region HashList: runs the base HashListMap constructor body by installing the HashListMap vtable, zeroing the count and list fields and memsets the 32 bucket pointers, then overwrites the vtab… |
| `00a455f0` | `HashList_RegionDefinition_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the derived region HashList: calls the map destructor 00a451b0 and frees the object with _free when bit 0 of the delete flag is set. |
