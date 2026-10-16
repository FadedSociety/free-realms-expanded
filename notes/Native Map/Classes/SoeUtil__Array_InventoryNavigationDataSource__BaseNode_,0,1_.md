# Class: SoeUtil::Array<InventoryNavigationDataSource::BaseNode*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7ab40` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the SoeUtil::Array<InventoryNavigationDataSource::BaseNode*,0,1> vtable (01816c00), with a count (arg 1), an out count pointer (arg 2) and an exact flag (arg 3): when the flag is 0 it keeps the old buf… |
| `00a7abe0` | `ArrayBaseNodePtr_Slot2_Free` | ✅ | virtual | Virtual slot 2 of the BaseNode pointer array vtable (01816c00), 18 bytes: loads the first stack argument, pushes 4 and that pointer, calls 00401c00 (FreeMemory with alignment 4), pops and returns with ret 8. |
| `00a7ac00` | `ArrayBaseNodePtr_Slot3_SwapOrForward` | ✅ | virtual | Virtual slot 3 of the BaseNode pointer array vtable (01816c00): when the flag argument is nonzero it forwards to slot 3 (+0xc) of the other array passed as the first argument; when the flag is zero it exchanges the data… |
| `00a7b050` | `ArrayBaseNodePtr_DtorBody` | ✅ | dtor | Destructor body (39 bytes): stores the BaseNode pointer array vtable at [this], sets the count at +8 to 0, frees the buffer at +4 through 00401c00 (alignment 4), nulls the pointer and returns. |
| `00a7b270` | `ArrayBaseNodePtr_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (56 bytes, vtable slot 0): stores the BaseNode pointer array vtable at [this], sets the count to 0, frees the buffer at +4 through 00401c00 (alignment 4), nulls it, calls _free (thunk 00d82463… |
