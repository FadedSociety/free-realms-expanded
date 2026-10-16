# Class: SoeUtil::Array<ItemCategoryDefinition_const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a7acb0` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the ItemCategoryDefinition const pointer array vtable (01816c14), 152 bytes, pseudocode identical to 00a7ab40: keeps the old buffer when the capacity is in range for the count, otherwise computes a new… |
| `00a7ad50` | `ArrayItemCategoryDefPtr_Slot2_Free` | ✅ | virtual | Virtual slot 2 of the ItemCategoryDefinition const pointer array vtable (01816c14), 18 bytes with the same shape as 00a7abe0: calls 00401c00 (FreeMemory, alignment 4) on the first stack argument and returns with ret 8. |
| `00a7ad70` | `ArrayItemCategoryDefPtr_Slot3_SwapOrForward` | ✅ | virtual | Virtual slot 3 of the ItemCategoryDefinition const pointer array vtable (01816c14); pseudocode identical to 00a7ac00: forwards to slot 3 (+0xc) of the other array when the flag is nonzero, otherwise swaps data pointer, … |
| `00a7b080` | `ArrayItemCategoryDefPtr_DtorBody` | ✅ | dtor | Destructor body (39 bytes): stores the ItemCategoryDefinition const pointer array vtable at [this] (0x01816c14), sets the count at +8 to 0, frees the buffer at +4 through 00401c00 (alignment 4), nulls the pointer and re… |
| `00a7b2b0` | `ArrayItemCategoryDefPtr_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (56 bytes, vtable slot 0): stores the ItemCategoryDefinition const pointer array vtable (0x01816c14) at [this], sets the count to 0, frees the buffer at +4 through 00401c00 (alignment 4), null… |
