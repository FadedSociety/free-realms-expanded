# Class: SoeUtil::Array<SoeUtil::StringFixed<64>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095ae70` | `— ⚠reject` | ⚪ | virtual | Chooses a capacity using the current count and request, returns existing storage when allowed, otherwise allocates capacity*0x54 bytes at alignment 4. |
| `0095af10` | `ArrayFreeStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4, then returns. |
| `0095af30` | `ArraySwapStorage` | ✅ | virtual | When param_3 is nonzero, invokes the indirect operation on param_2; otherwise swaps the storage pointer and exchanges the two size and capacity fields. |
| `0095e230` | `ArrayDestructor → SoeUtil::Array<SoeUtil::StringFixed<64>,0,1>::~Array` | ✅ | dtor | Restores the array vtable, destroys embedded elements with FUN_009594c0, releases backing storage at alignment 4, and clears the storage pointer. |
| `0095f160` | `ArrayScalarDeletingDtor → SoeUtil::Array<SoeUtil::StringFixed<64>,0,1>::ScalarD…` | ✅ | virtual | Restores the array vtable, destroys its elements, releases backing storage, and frees the object when the deleting flag is set. |
| `0096f360` | `StringFixed64ArrayDtor` | ✅ | dtor | Releases the buffers for the supplied array and this array, destroys both array descriptors, then frees a final zero-sized aligned allocation. |
