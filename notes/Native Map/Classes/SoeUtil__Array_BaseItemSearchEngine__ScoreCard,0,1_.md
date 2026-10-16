# Class: SoeUtil::Array<BaseItemSearchEngine::ScoreCard,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2e0a0` | `ArrayResizeAllocator` | ✅ | method | Chooses capacity, returns existing storage or null when applicable, otherwise allocates 0xc bytes per ScoreCard slot and writes the capacity. |
| `00c2e140` | `ArrayFreeBuffer` | ✅ | method | Calls FUN_00401c00(pointer,4) on the supplied argument and returns with eight stack bytes callee-cleaned. |
| `00c2e160` | `ArraySwapStorage` | ✅ | method | With flag set dispatches through param_2 slot +0xc; otherwise swaps data pointers then XOR-swaps size and capacity fields at +8/+0xc. |
| `00c2e9c0` | `ScoreCardArrayDtor` | ✅ | dtor | Restores the array vtable, zeros count, frees its data pointer with alignment 4, and clears the pointer. |
| `00c2eea0` | `ScoreCardArrayScalarDeletingDtor` | ✅ | dtor | Restores the array vtable, frees and clears the backing pointer, and calls _free(this) when flag bit 0 is set. |
