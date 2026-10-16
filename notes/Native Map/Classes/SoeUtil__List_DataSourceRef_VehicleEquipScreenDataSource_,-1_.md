# Class: SoeUtil::List<DataSourceRef<VehicleEquipScreenDataSource>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0098fcd0` | `— ⚠reject` | ⚪ | method | Runs the list allocator cleanup, releases an available allocator object, or invokes the allocation failure path when none is available. |
| `0098fd10` | `VehicleEquipDataSourceListFreeMemory` | ✅ | method | Pushes arguments for four-byte alignment and calls the shared memory-free helper, then returns. |
| `00991550` | `VehicleEquipDataSourceListDtor` | ✅ | dtor | Installs the vehicle equip-screen list vtable and clears all nodes from the list. |
| `00991600` | `VehicleEquipDataSourceListScalarDeletingDtor` | ✅ | dtor | Installs the list vtable, clears list nodes, conditionally frees the object when the deleting flag bit is set, and returns the pointer otherwise. |
