# Class: SoeUtil::List<LuaCallDetails,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be220` | `LuaCallDetailsList::AllocateNode` | ✅ | virtual | Allocates a 0x4c-byte node aligned to 4 after trying the thread and global allocators. |
| `008be260` | `LuaCallDetailsList::FreeNode` | ✅ | virtual | Releases the supplied node with 4-byte alignment. |
| `00919960` | `SoeUtil::ListLuaCallDetailsDtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the List vtable, then loops over the chain at +4 calling FUN_00909520 as a thiscall on the container with the node pushed. |
| `0091a600` | `SoeUtil::ListLuaCallDetailsScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_00909520 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
