# Class: SoeUtil::Array<ClientLuaParam,7,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0071b8a0` | `ClientLuaParamInline7Free` | ✅ | virtual | Frees the supplied block with alignment 8 unless it equals (this+0x17)&~7; for an 8-byte-aligned object address this expression resolves to this+0x10. |
| `0071ca00` | `ClientLuaParamInline7Destructor` | ✅ | dtor | Installs inline-seven vtable, destroys elements, frees external storage when present, then restores and clears zero-capacity base array. |
| `0071caa0` | `ClientLuaParamInline7ChooseCapacity` | ✅ | virtual | Returns inline storage and capacity seven for requests up to seven; delegates larger requests to ArrayGrow. |
| `0071cc20` | `ClientLuaParamInline7Constructor` | ✅ | ctor | Initializes zero-capacity base array and installs inline-seven specialization vtable. |
| `0071cca0` | `ClientLuaParamInline7ScalarDeletingDtor` | ✅ | dtor | Calls inline-seven array destructor and frees object when deleting flag bit is set. |
| `0071cdc0` | `ClientLuaParamArrayOperation7` | ✅ | virtual | For each of two array objects, reads its count and backing pointer, calls 00709410 with that pair, then calls 0070b4e0 with its count. It then calls those helpers with zero values and frees an 8-aligned pointer; exact e… |
