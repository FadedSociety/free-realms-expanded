# Class: SoeUtil::Array<SensoryEffectDefinition*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095afc0` | `PointerArrayAllocateStorage` | ✅ | virtual | Computes a replacement capacity and allocates capacity*4 bytes at alignment 4, or returns the existing pointer when the capacity rules allow it. |
| `0095b060` | `PointerArrayFreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4, then returns. |
| `0095b080` | `PointerArraySwapStorage` | ✅ | virtual | When param_3 is nonzero, invokes the indirect operation on param_2; otherwise swaps the data pointer and exchanges size and capacity fields. |
| `0095e360` | `ArrayDestructor → SoeUtil::Array<SensoryEffectDefinition*,0,1>::~Array` | ✅ | dtor | Restores the pointer-array vtable, clears its size, frees the backing pointer at alignment 4, and zeros the storage field. |
| `0095f1a0` | `ArrayScalarDeletingDtor → SoeUtil::Array<SensoryEffectDefinition*,0,1>::ScalarD…` | ✅ | virtual | Restores the pointer-array vtable, clears count, releases backing storage, and conditionally frees the object. |
