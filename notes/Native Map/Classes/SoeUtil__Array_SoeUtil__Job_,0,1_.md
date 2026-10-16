# Class: SoeUtil::Array<SoeUtil::Job*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d0b510` | `SoeUtil::Array::AllocateCapacity` | ✅ | virtual | Selects target capacity using growth/shrink thresholds, returns current buffer when unchanged, otherwise allocates capacity times four bytes. |
| `00d0b5b0` | `SoeUtil::Array::FreeStorage` | ✅ | virtual | Passes supplied array storage pointer to memory manager with four-byte alignment. |
| `00d0b5d0` | `SoeUtil::Array::SwapStorage` | ✅ | virtual | When flag set, dispatches virtual slot +0xc on supplied object; otherwise swaps buffer pointers and the two integer array fields. |
| `00d0b7f0` | `SoeUtil::Array::Array ? ⚠rename` | 🟡 | ctor | Installs array vtable, clears its size, frees old backing field through memory manager, and resets pointer. |
| `00d0b890` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | virtual | Installs array vtable, frees and clears backing storage, then frees object when requested. |
| `00d0bf70` | `ClearJobPointerArrays ? ⚠rename` | 🟡 | dtor | Releases referenced Job pointers in first array, resets both embedded array vtables and sizes, and frees both backing allocations. |
