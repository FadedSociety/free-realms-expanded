# Class: SoeUtil::Array<SoeUtil::StrongType<int,BaseItemSearchEngine::SoeUtilSignatureKey,1>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009900a0` | `— ⚠reject` | ⚪ | method | Chooses a capacity from the requested count and current capacity, returns the existing buffer when capacity is sufficient, or allocates count-times-four bytes aligned to four. |
| `00990140` | `SignatureKeyArrayFreeMemory` | ✅ | method | Calls the shared free helper with the supplied pointer and alignment four, then returns. |
| `00990160` | `SignatureKeyArrayExchange` | ✅ | virtual | When the flag is set, invokes the source object's virtual slot +0x0c; otherwise swaps the backing pointer and uses XOR swaps for the following two dwords when unequal. |
| `00990680` | `SignatureKeyArrayDtor` | ✅ | dtor | Restores the array vtable, clears its size, frees its backing buffer with four-byte alignment, and nulls the buffer pointer. |
| `009907a0` | `SignatureKeyArrayScalarDeletingDtor` | ✅ | dtor | Resets the array vtable, clears size, frees its backing buffer, and frees the object itself when the deleting flag is set. |
