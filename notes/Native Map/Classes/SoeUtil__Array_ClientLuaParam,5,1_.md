# Class: SoeUtil::Array<ClientLuaParam,5,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be880` | `ClientLuaParamArray5::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e14c0` | `ArrayClientLuaParam5_Dtor` | ✅ | dtor | Installs the five-inline-element vtable, destroys used LuaParam entries, frees non-inline storage, then restores the generic array base and clears storage. |
| `008e1560` | `ArrayClientLuaParam5_InlineCapacity` | ✅ | virtual | Returns aligned inline storage with capacity 5 when request fits; forwards larger requests to the common grow helper. |
| `008e1590` | `ArrayClientLuaParam5_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f4f40` | `ArrayClientLuaParam5_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-five array vtable. |
