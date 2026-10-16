# Class: SoeUtil::List<PlayerCustomizationData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcca0` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x24 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bcce0` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `009197f0` | `SoeUtil::ListPlayerCustomizationDataDtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the List vtable, then loops over the chain at +4 calling FUN_008fbe20 as a thiscall on the container with the node pushed. |
| `0091a440` | `SoeUtil::ListPlayerCustomizationDataScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_008fbe20 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
