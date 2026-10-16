# Class: SoeUtil::Array<ClientLuaParam,6,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be2b0` | `ClientLuaParamArray6::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e1170` | `ArrayClientLuaParam6_Dtor` | ✅ | dtor | Installs the six-inline-element vtable, destroys used LuaParam entries, frees non-inline storage, restores the generic array vtable, and clears the generic storage. |
| `008e1210` | `ArrayClientLuaParam6_InlineCapacity` | ✅ | virtual | Returns aligned inline storage and capacity 6 for requests at most six; delegates larger requests to the shared grow helper. |
| `008e1240` | `ArrayClientLuaParam6_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f4ee0` | `ArrayClientLuaParam6_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-six array vtable. |
