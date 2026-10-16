# Class: SoeUtil::Array<ClientLuaParam,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708950` | `ArrayGrow → ClientLuaParamArrayGrow` | ✅ | virtual | Chooses whether to reuse or resize storage from requested and current counts, updates the output capacity, and allocates capacity times 0xa0 bytes aligned to 8. |
| `007089f0` | `ArrayFree` | ✅ | virtual | Passes the supplied array pointer and alignment 8 to the memory-manager free routine. |
| `00708a10` | `ArraySwapStorage` | ✅ | virtual | When the flag is set, calls the supplied object's virtual method; otherwise swaps the data pointer and exchanges the two count/capacity fields. |
| `0070a6d0` | `ArrayAssign → ClientLuaParamArrayAssign` | ✅ | method | Destroys any active array variant, sets tag 6, initializes the embedded array state, and copies elements from the source using 00709410. |
| `0070e110` | `ClientLuaParamArrayDestructor → SoeUtil::Array<ClientLuaParam,0,1>::Destructor` | ✅ | dtor | Restores zero-inline array vtable, clears elements, frees aligned storage, and zeros its pointer. |
| `0070e3b0` | `ClientLuaParamArrayScalarDeletingDtor` | ✅ | dtor | Clears elements and storage; returns the original object when (param2 & 1) == 0 and frees it when bit 0 is set. |
| `0070ed30` | `ClientLuaParamArrayAssign` | 🟢 | virtual | Passes source data/count to array growth, destroys elements through the cleanup helper, repeats for destination storage, then cleans temporary array state. |
| `0070ee30` | `ClientLuaParamArrayAssign → ClientLuaParamArrayInline1Assign` | 🟢 | virtual | Passes source data/count to array growth, destroys elements through the cleanup helper, repeats for destination storage, then cleans temporary array state. |
| `008ebdd0` | `— ⚠reject` | ⚪ | ctor | Initializes the destination fixed-string member and ClientLuaParam array vtable, zeros array count/capacity fields, and assigns the supplied name. If a source array is present, destroys existing active elements, reads s… |
| `00a196e0` | `ArrayAssignAndRelease ?` | 🟡 | method | Resizes and clears both arrays using the 0xa0-byte element routines, creates and destroys a temporary empty array, then frees aligned storage. |
| `00a8d3a0` | `ClientLuaParamArrayDestructor` | ✅ | dtor | Restores the array vtable, destroys live ClientLuaParam elements, frees its allocated storage with alignment 8, restores the IString vtable, and releases the owned string buffer. |
