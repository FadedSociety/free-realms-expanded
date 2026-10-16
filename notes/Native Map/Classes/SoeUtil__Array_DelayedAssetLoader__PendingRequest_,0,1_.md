# Class: SoeUtil::Array<DelayedAssetLoader::PendingRequest*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ae3e20` | `PendingRequestArrayAllocate` | ✅ | virtual | Chooses capacity from old/new counts and growth mode, writes selected capacity through output pointer, returns existing storage or allocates count*4 bytes aligned to 4. |
| `00ae3ec0` | `PendingRequestArrayFree` | ✅ | virtual | Calls FreeMemory with supplied pointer and alignment 4, then returns. |
| `00ae3ee0` | `— ⚠reject` | ⚪ | virtual | When release flag is set calls pending object's virtual release slot; otherwise swaps array pointer/size/capacity fields with supplied object fields. |
| `00ae4510` | `PointerArrayDtor → PendingRequestPointerArrayDtor` | ✅ | dtor | Installs array vtable, clears length, frees backing buffer with alignment 4, and nulls data pointer. |
| `00ae4630` | `PointerArray::ScalarDeletingDtor` | ✅ | dtor | Clears array vtable and length, frees dynamic backing with alignment 4, and frees this when flag bit 0 is set. |
