# Class: SoeUtil::Array<ClientLuaParam,8,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008be8a0` | `ClientLuaParamArray8::FreeBuffer` | ✅ | virtual | Preserves the inline storage at this+0x17 aligned to 8; frees non-inline buffers with alignment 8. |
| `008e1690` | `ArrayClientLuaParam8_Dtor` | ✅ | dtor | Installs the eight-inline-element vtable, destroys active elements, frees heap storage when separate from inline storage, and clears the generic base array. |
| `008e1730` | `ArrayClientLuaParam8_InlineCapacity` | ✅ | virtual | Returns aligned inline storage and capacity eight for small requests, delegating larger requests to the shared grow helper. |
| `008e1760` | `ArrayClientLuaParam8_SwapContents` | ✅ | virtual | Uses ClientLuaParam array helpers to copy the argument into a temporary array, exchange the two arrays' contents, then release temporary storage with eight-byte alignment. |
| `008f4fa0` | `ArrayClientLuaParam8_Ctor` | ✅ | ctor | Installs the generic ClientLuaParam array vtable, zeros its three state dwords, then installs the inline-capacity-eight array vtable. |
