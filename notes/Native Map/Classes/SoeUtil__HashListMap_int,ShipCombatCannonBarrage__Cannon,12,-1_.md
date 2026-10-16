# Class: SoeUtil::HashListMap<int,ShipCombatCannonBarrage::Cannon,12,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cae180` | `— ⚠reject` | ⚪ | ctor | Installs the HashListMap vtable, clears counters and 12 bucket slots, initializes allocator state, and returns the object. |
| `00caedf0` | `HashListMapDtorBody` | ✅ | dtor | Stores the HashListMap vtable and calls the map clear routine. |
| `00caee00` | `HashListMap::ScalarDeletingDtor` | ✅ | dtor | Calls map destructor body and frees allocation when the scalar deleting flag's low bit is set. |
| `00caee90` | `HashListMapClear` | ✅ | method | Installs the map vtable for cleanup and calls the shared remove-all routine. |
| `00caefe0` | `HashListMap::Ctor ⚠downgrade` | 🟢 | ctor | Installs the HashListMap vtable, clears counters and bucket storage, and initializes list state. |
| `00caf060` | `HashListMap::ScalarDeletingDtorBase` | ✅ | dtor | Calls map clear routine and frees allocation when scalar deleting flag's low bit is set. |
