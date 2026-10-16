# Class: SoeUtil::Array<ClientLuaParam,20,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a188c0` | `Array_ClientLuaParam_Dtor ⚠downgrade` | 🟢 | dtor | Destructor of the Lua-parameter array: stores the 20-inline-element vftable, destroys the live elements through FUN_0070b4e0 (which releases the variant payloads of the 0xa0-byte ClientLuaParam records), frees the heap … |
| `00a18960` | `Array_ClientLuaParam_BufferHook` | ✅ | virtual | Virtual slot 1 of the Lua-parameter array: tail-jumps to the generic grow routine 00708950 when the requested count exceeds 0x14, otherwise reports capacity 0x14 and returns the inline storage (this+0x17)&0xfffffff8 for… |
| `00a18b30` | `Array_ClientLuaParam::ScalarDeletingDtor` | 🟢 | dtor | Calls the array destructor at 00a188c0 and frees the object when the deleting flag is set. |
| `00a8d810` | `ClientLuaParamInline20ArrayConstructor` | ✅ | ctor | Zeros the array data, count, and capacity, then installs the 20-inline-element specialization vtable. |
