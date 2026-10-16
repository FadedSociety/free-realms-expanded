# Class: SoeUtil::Array<BaseItemSearchEngine::SearchField,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e410` | `ArrayResizeAllocator` | ✅ | method | Chooses SearchField capacity, may return current storage or null, otherwise allocates 0x218 bytes per slot at alignment 4. |
| `00c2e4b0` | `ArrayFreeBuffer` | ✅ | method | Frees the supplied array buffer by calling FUN_00401c00 with alignment 4. |
| `00c2e4d0` | `ArraySwapStorage` | ✅ | method | Dispatches through param_2 slot +0xc when requested; otherwise swaps backing pointers and XOR-swaps size and capacity at +8/+0xc. |
| `00c30640` | `SearchFieldArrayDtor` | ✅ | dtor | Restores the array vtable, destroys counted elements through FUN_00c303d0, frees its backing pointer at alignment 4, and clears the pointer. |
| `00c30820` | `SearchFieldArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, clears elements, frees backing storage, and calls _free(this) when flag bit 0 is set. |
