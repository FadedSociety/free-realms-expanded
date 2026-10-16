# Class: SoeUtil::Array<ClientLuaParam,13,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25520` | `SoeUtil::ArrayClientLuaParam13FreeStorage` | ✅ | virtual | Virtual free slot (vtable index 2) of the 13-element inline array: frees with 8-byte alignment unless the pointer equals the inline buffer at (this+0x17) & ~7. |
| `00b30df0` | `ClientLuaParamArray13::Dtor` | ✅ | dtor | Destroys ClientLuaParam elements, frees external array storage when it is not inline, clears the storage pointer, and restores the zero-capacity array vtable. |
| `00b30e90` | `— ⚠reject` | ⚪ | virtual | For requested counts at most 13, returns aligned inline storage at this+0x17 and writes capacity 13; larger requests call 00708950 and return its result. |
| `00b34710` | `ClientLuaParamArray13::Construct` | ✅ | ctor | Installs base zero-inline array vtable, zeros data/count/capacity, then installs the 13-inline specialization vtable. |
| `00b34870` | `ClientLuaParamArray13::ScalarDeletingDtor` | ✅ | dtor | Calls array destructor; when flag bit 0 is set frees object, otherwise returns original pointer. |
| `00b34c60` | `ClientLuaParamArray13::CopyAndRelease ? ⚠rename` | 🟡 | method | Reserves matching capacity and initializes elements for both arrays, destroys their used elements, then releases the temporary empty array and storage. |
