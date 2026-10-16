# Class: SoeUtil::Array<ClientLuaParam,2,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00708b50` | `InlineArrayFree → ClientLuaParamInline2ArrayFree` | ✅ | virtual | Compares the pointer with the aligned inline buffer address and frees it with alignment 8 only when it is external storage. |
| `0070e170` | `ClientLuaParamArrayInline2Destructor` | ✅ | dtor | Destroys elements, frees non-inline storage, then restores and cleans the zero-capacity base array. |
| `0070e210` | `ClientLuaParamArrayInline2Capacity` | ✅ | virtual | Returns inline buffer for requested capacity at most two; larger requests jump to shared allocation helper. |
| `0070ebe0` | `ClientLuaParamArrayInline2Ctor` | ✅ | ctor | Zeros array data/count/capacity, installs base array vtable, then installs two-inline specialization vtable. |
| `0070eca0` | `ClientLuaParamArrayInline2ScalarDtor` | ✅ | dtor | Calls the two-slot array destructor; returns the original object when (param2 & 1) == 0 and frees it when bit 0 is set. |
