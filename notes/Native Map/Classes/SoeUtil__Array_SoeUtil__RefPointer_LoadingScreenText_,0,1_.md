# Class: SoeUtil::Array<SoeUtil::RefPointer<LoadingScreenText>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca8920` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the text array: same growth-or-exact reservation as ArrayRefLoadingScreenData_Reserve, allocating count*4 bytes with 4-byte alignment through 00401bb0 and writing the capacity through the output pointe… |
| `00ca89c0` | `ArrayRefLoadingScreenText_FreeBuffer ⚠downgrade` | 🟢 | virtual | Virtual slot 2 of the text array: frees the buffer argument through FreeMemory with alignment 4 (00401c00) and returns, popping 8 bytes. |
| `00ca89e0` | `ArrayRefLoadingScreenText_SwapState` | ✅ | virtual | Virtual slot 3 of the text array: tail-dispatches to the other object's vtable slot 3 when the flag argument is non-zero, otherwise exchanges the words at +4, +8 and +0xc with the other array object. |
| `00ca96a0` | `ArrayRefLoadingScreenText_Dtor` | ✅ | dtor | Destructor body of the LoadingScreenText pointer array: installs its vtable, truncates all entries through RefPointerArray_TruncateB (00ca8f80), frees the buffer through FreeMemory (00401c00) and clears the buffer point… |
| `00ca9710` | `ArrayRefLoadingScreenText_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the LoadingScreenText pointer array (vtable slot 0): installs the vtable, truncates via 00ca8f80, frees the buffer through 00401c00, clears it, and frees this through _free when bit 0 of th… |
