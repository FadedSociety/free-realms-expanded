# Class: SoeUtil::Array<ItemSearchToken,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e1f0` | `ArrayResizeAllocator` | ✅ | method | Selects new capacity or existing storage as applicable, then allocates param_2 times 0x410d4 bytes at alignment 4. |
| `00c2e290` | `ArrayFreeBuffer` | ✅ | method | Calls FUN_00401c00(pointer,4) on its argument and returns with eight stack bytes callee-cleaned. |
| `00c2e2b0` | `ArraySwapStorage` | ✅ | method | When flag is set dispatches through param_2 slot +0xc; otherwise swaps backing pointers and XOR-swaps size and capacity. |
| `00c2f260` | `ItemSearchTokenArrayDtor` | ✅ | dtor | Restores the token-array vtable, clears counted elements, frees the buffer at alignment 4, and nulls its pointer. |
| `00c2f2b0` | `ItemSearchTokenArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears elements, frees its data, and frees the object only when flag bit 0 is set. |
