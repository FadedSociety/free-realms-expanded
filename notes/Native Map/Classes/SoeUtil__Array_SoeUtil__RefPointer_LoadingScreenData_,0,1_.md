# Class: SoeUtil::Array<SoeUtil::RefPointer<LoadingScreenData>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca8780` | `— ⚠reject` | ⚪ | virtual | Virtual slot 1 of the array: when the flag argument is 0 it applies a growth rule (keeps the buffer when capacity is adequate, otherwise grows by 5/4 or 6/5), when non-zero it sizes exactly; writes the new capacity thro… |
| `00ca8820` | `ArrayRefLoadingScreenData_FreeBuffer` | ✅ | virtual | Virtual slot 2 of the array: frees the buffer argument through FreeMemory with alignment 4 (00401c00) and returns, popping 8 bytes. |
| `00ca8840` | `ArrayRefLoadingScreenData_SwapState` | ✅ | virtual | Virtual slot 3 of the array: when the flag argument is non-zero it tail-dispatches to the other object's vtable slot 3; otherwise it exchanges the words at +4, +8 and +0xc with the other array object (xor-swap style). |
| `00ca9670` | `ArrayRefLoadingScreenData_Dtor` | ✅ | dtor | Destructor body of the LoadingScreenData pointer array: installs its vtable, truncates all entries through RefPointerArray_TruncateA (00ca8f10), frees the buffer through FreeMemory (00401c00) and clears the buffer point… |
| `00ca96d0` | `ArrayRefLoadingScreenData_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the LoadingScreenData pointer array (vtable slot 0): installs the vtable, truncates via 00ca8f10, frees the buffer through 00401c00, clears it, and frees this through _free when bit 0 of th… |
