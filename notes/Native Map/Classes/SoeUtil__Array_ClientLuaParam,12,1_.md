# Class: SoeUtil::Array<ClientLuaParam,12,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bea30` | `ClientLuaParamArray12::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e1c00` | `ArrayClientLuaParam12_Dtor` | ✅ | dtor | Installs the twelve-inline-element vtable, destroys active elements, frees non-inline storage, and restores/clears the generic array base. |
| `008e1ca0` | `ArrayClientLuaParam12_InlineCapacity` | ✅ | virtual | Returns aligned inline storage and capacity 12 when the request fits, otherwise uses the shared grow routine. |
| `008e1cd0` | `ArrayClientLuaParam12_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f50c0` | `ArrayClientLuaParam12_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-twelve array vtable. |
