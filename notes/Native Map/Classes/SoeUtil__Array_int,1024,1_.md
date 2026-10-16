# Class: SoeUtil::Array<int,1024,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b24780` | `SoeUtil::ArrayInt1024FreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2) of the 1024-int inline array: frees the pointer argument unless it equals the inline buffer at (this+0x13) & ~3. |
| `00b285a0` | `SoeUtil::ArrayInt1024Dtor` | ✅ | dtor | Destructor: writes the Array<int,1024,1> vtable, clears the size at +8, frees +4 only when it is not the inline buffer ((this+0x13) & ~3), then zeros +4. It writes the base Array<int,0,1> vtable, clears +8, calls FUN_00… |
| `00b28630` | `SoeUtil::ArrayInt1024AllocSlot` | ✅ | virtual | Virtual allocate slot (vtable index 1) of the inline array: when the requested count exceeds the 0x400 inline capacity it tail-jumps to FUN_0073a540; otherwise it reports capacity 0x400 through param_3 and returns the i… |
| `00b2a8e0` | `ArrayInt1024::ScalarDeletingDtor` | ✅ | virtual | Calls the array destructor, then frees the object if the deleting flag's low bit is set; otherwise returns its pointer. |
