# Class: SoeUtil::HashListMap<int,GodBeamDefinition,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009fdc00` | `— ⚠reject` | ⚪ | method | Runs allocator setup, obtains a pool allocator if available, otherwise calls the global allocator for a 0x2c-byte block aligned to 4, and returns the allocation. |
| `009fdc40` | `GodBeamMapFreeBlock` | ✅ | method | Calls FUN_00401c00 to release the supplied block with alignment value 4. |
| `009fe4b0` | `GodBeamMapDtor` | ✅ | dtor | Installs the map vtable, then repeatedly removes nodes at field +8 through the node-removal helper until empty. |
| `009fe4e0` | `— ⚠reject` | ⚪ | dtor | Installs the hash-list map vtable, removes every remaining node, frees the map when flag bit 0 is set, and otherwise returns it. |
| `009fe520` | `GodBeamMapClearAndDtor` | ✅ | dtor | Installs both -1 and 1 sentinel vtables, removes all nodes, frees its auxiliary block tree, zeros pool fields, then restores the -1 specialization vtable and clears its nodes again. |
