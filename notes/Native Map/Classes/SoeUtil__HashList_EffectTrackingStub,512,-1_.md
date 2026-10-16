# Class: SoeUtil::HashList<EffectTrackingStub,512,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095b7b0` | `HashList::AllocateNode` | ✅ | virtual | Initializes allocator state, obtains the current allocator, and calls its virtual slot +4 if present; otherwise it tries the global allocator at DAT_01bdf280 through slot +4. Only when neither allocator is available doe… |
| `0095b7f0` | `HashList::FreeNode` | ✅ | virtual | Passes the supplied node pointer and alignment 8 to FUN_00401c00. |
| `00964090` | `EffectTrackingStubHashListCtor` | ✅ | ctor | Installs the map vtable, clears three bookkeeping fields, zeroes 0x800 bytes of bucket storage, clears another field, and installs the typed HashList vtable. |
| `00964110` | `EffectTrackingStubHashListScalarDeletingDestructor` | ✅ | dtor | Calls FUN_00962e80 to destroy the typed list; if param_2 bit 0 is set, frees param_1; otherwise returns the pointer. |
