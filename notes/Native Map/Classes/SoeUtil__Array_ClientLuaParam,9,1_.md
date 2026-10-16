# Class: SoeUtil::Array<ClientLuaParam,9,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be8c0` | `ClientLuaParamArray9::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e1860` | `ArrayClientLuaParam9_Dtor` | ✅ | dtor | Installs the nine-inline-element vtable, destroys active entries, frees heap storage when distinct from inline storage, and clears the generic array base. |
| `008e1900` | `ArrayClientLuaParam9_InlineCapacity` | ✅ | virtual | Returns inline storage and capacity nine when the request fits, otherwise delegates to the common grow helper. |
| `008e1930` | `ArrayClientLuaParam9_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f5000` | `ArrayClientLuaParam9_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-nine array vtable. |
