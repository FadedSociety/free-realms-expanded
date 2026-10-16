# Class: SoeUtil::Array<ClientLuaParam,1,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `007090f0` | `InlineArrayFree` | ✅ | virtual | Compares the pointer with the aligned inline buffer address and frees it with alignment 8 only when it is external storage. |
| `0070e2e0` | `ClientLuaParamArrayInline1Destructor` | ✅ | dtor | Destroys elements, frees non-inline storage, then restores and cleans the zero-capacity base array. |
| `0070e380` | `ClientLuaParamArrayInline1Capacity` | ✅ | virtual | Returns inline buffer for requested capacity at most one; larger requests jump to shared allocation helper. |
| `0070ec40` | `ClientLuaParamArrayInline1Ctor` | ✅ | ctor | Zeros array data/count/capacity, installs base array vtable, then installs one-inline specialization vtable. |
| `0070ecc0` | `ClientLuaParamArrayInline1ScalarDtor` | ✅ | dtor | Calls the one-slot array destructor; returns the original object when (param2 & 1) == 0 and frees it when bit 0 is set. |
