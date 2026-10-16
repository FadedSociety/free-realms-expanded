# Class: SoeUtil::Array<ClientLuaParam,3,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071b880` | `ClientLuaParamInline3Free` | ✅ | virtual | Frees supplied block with alignment eight unless it equals aligned inline storage at object offset 0x17. |
| `0071c930` | `ClientLuaParamInline3Destructor` | ✅ | dtor | Installs inline-three vtable, destroys elements, frees external storage when present, then restores and clears zero-capacity base array. |
| `0071c9d0` | `ClientLuaParamInline3ChooseCapacity` | ✅ | virtual | Returns inline storage and capacity three for requests up to three; delegates larger requests to ArrayGrow. |
| `0071cbc0` | `ClientLuaParamInline3Constructor` | ✅ | ctor | Initializes zero-capacity base array and installs inline-three specialization vtable. |
| `0071cc80` | `ClientLuaParamInline3ScalarDeletingDtor` | ✅ | dtor | Calls inline-three array destructor and frees object when deleting flag bit is set. |
| `0071ccc0` | `ClientLuaParamArrayOperation3` | ✅ | virtual | For each of two array objects, reads its count and backing pointer, calls 00709410 with that pair, then calls 0070b4e0 with its count. It then calls those helpers with zero values and frees an 8-aligned pointer; exact e… |
