# Class: SoeUtil::Array<ShipOrientation,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca43a0` | `Array_AllocateStorage` | ✅ | virtual | Chooses growth or shrink capacity from the current and requested counts, returns existing storage when allowed, otherwise allocates capacity times 32 bytes at alignment 0x10. |
| `00ca4440` | `Array_FreeStorage` | ✅ | virtual | Frees the supplied array storage using alignment 0x10. |
| `00ca4460` | `Array_SwapStorage` | ✅ | virtual | Either delegates to the supplied object's virtual method when the flag is set, or swaps the array data pointer and exchanges two metadata words. |
| `00ca45d0` | `— ⚠reject` | ⚪ | method | Grows the array when the requested index exceeds current size, updates the element count, initializes each new 32-byte element with defaults, and returns the requested element address. |
| `00ca48e0` | `Array_Dtor → SoeUtil::Array<ShipOrientation,0,1>::Dtor` | ✅ | dtor | Restores the array vtable, clears its count, frees backing storage at alignment 0x10, and clears the data pointer. |
| `00ca4910` | `Array_ScalarDeletingDtor → SoeUtil::Array<ShipOrientation,0,1>::ScalarDeletingD…` | ✅ | dtor | Restores the array vtable, clears its count and storage pointer, frees backing storage, and optionally frees the array object itself. |
| `00ca4c30` | `Array_GetElement` | 🟢 | method | Optionally initializes a just-appended element, then returns the backing pointer plus index times the 0x20-byte element stride. |
