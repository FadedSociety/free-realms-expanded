# Class: SoeUtil::Array<ClientLuaParam,15,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b25540` | `SoeUtil::ArrayClientLuaParam15FreeStorage ? ⚠downgrade` | 🟡 | virtual | Virtual free slot (vtable index 2) of the 15-element inline array: frees with 8-byte alignment unless the pointer equals the inline buffer at (this+0x17) & ~7. |
| `00b30ec0` | `ClientLuaParamArray15::Dtor` | ✅ | dtor | Destroys ClientLuaParam elements, releases non-inline storage, clears the pointer, and restores the zero-capacity array vtable. |
| `00b30f60` | `ClientLuaParamArray15::Allocate` | ✅ | virtual | For requested counts at most 15, returns aligned inline storage at this+0x17 and writes capacity 15; larger requests call 00708950 and return its result. |
| `00b34770` | `ClientLuaParamArray15::Construct` | ✅ | ctor | Installs base zero-inline array vtable, zeros data/count/capacity, then installs the 15-inline specialization vtable. |
| `00b34890` | `ClientLuaParamArray15::ScalarDeletingDtor` | ✅ | dtor | Calls array destructor; when flag bit 0 is set frees object, otherwise returns original pointer. |
| `00b34d60` | `ClientLuaParamArray15::CopyAndRelease ⚠downgrade` | 🟢 | method | Reserves matching capacity and initializes elements for both arrays, destroys their used elements, then releases the temporary empty array and storage. |
