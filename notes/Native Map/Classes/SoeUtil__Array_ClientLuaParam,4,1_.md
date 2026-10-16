# Class: SoeUtil::Array<ClientLuaParam,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be280` | `ClientLuaParamArray4::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e0fa0` | `ArrayClientLuaParam4_Dtor` | ✅ | dtor | Installs the four-inline-element array vtable, destroys used Lua-parameter entries, frees heap storage when distinct from inline storage, then restores and clears the generic array base. |
| `008e1040` | `ArrayClientLuaParam4_InlineCapacity` | ✅ | virtual | Returns the object's aligned inline storage and reports capacity 4 when requested count is at most four; otherwise forwards to the shared grow helper. |
| `008e1070` | `ArrayClientLuaParam4_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f4e80` | `ArrayClientLuaParam4_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-four array vtable. |
