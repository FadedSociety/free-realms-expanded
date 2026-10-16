# Class: SoeUtil::List<DataSourceRef<EquipScreenDataSource>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098fc10` | `— ⚠reject` | ⚪ | method | Runs the list allocator cleanup, releases the selected allocator object when present, and calls the allocation failure path if neither allocator is available. |
| `0098fc50` | `EquipDataSourceListFreeMemory` | ✅ | method | Pushes arguments for four-byte alignment and calls the shared memory-free helper, then returns. |
| `00991540` | `EquipDataSourceListDtor` | ✅ | dtor | Installs the equip-screen list vtable and clears all nodes from the list. |
| `009915d0` | `EquipDataSourceListScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, clears list nodes, conditionally frees the object when the deleting flag bit is set, and returns the pointer otherwise. |
