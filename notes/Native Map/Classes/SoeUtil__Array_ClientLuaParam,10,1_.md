# Class: SoeUtil::Array<ClientLuaParam,10,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bea10` | `ClientLuaParamArray10::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e1a30` | `ArrayClientLuaParam10_Dtor` | ✅ | dtor | Installs the ten-inline-element vtable, destroys active entries, frees non-inline storage, then clears generic-array storage. |
| `008e1ad0` | `ArrayClientLuaParam10_InlineCapacity` | ✅ | virtual | Returns aligned inline storage and capacity ten for requests up to ten, otherwise delegates to the shared grow helper. |
| `008e1b00` | `ArrayClientLuaParam10_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f5060` | `ArrayClientLuaParam10_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-ten array vtable. |
