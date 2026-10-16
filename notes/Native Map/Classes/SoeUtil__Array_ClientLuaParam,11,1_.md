# Class: SoeUtil::Array<ClientLuaParam,11,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0e3e0` | `Array::FreeBuffer → SoeUtil::Array<ClientLuaParam,11,1>::FreeBuffer` | ✅ | virtual | Frees the supplied buffer with alignment 8 unless it matches the aligned in-object small-buffer address. |
| `00b10490` | `ClientLuaParamArray::Dtor` | ✅ | dtor | Restores inline and dynamic array vtables, destroys live ClientLuaParam elements, and releases backing storage. |
| `00b10530` | `ClientLuaParamArray::AllocateBuffer` | ✅ | virtual | Returns aligned inline storage for capacities up to eleven, otherwise delegates to the dynamic allocator. |
| `00b107a0` | `ClientLuaParamArray::Ctor` | ✅ | ctor | Initializes dynamic storage metadata and installs the inline ClientLuaParam array vtable. |
| `00b10800` | `ClientLuaParamArray::ScalarDeletingDtor` | ✅ | dtor | Calls the inline ClientLuaParam array destructor and conditionally frees the object. |
| `00b10840` | `— ⚠reject` | ⚪ | virtual | Copies the source array's active elements to temporary storage, destroys active elements in both arrays, then releases temporary storage. |
