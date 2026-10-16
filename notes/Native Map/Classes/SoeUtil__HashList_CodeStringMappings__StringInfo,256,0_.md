# Class: SoeUtil::HashList<CodeStringMappings::StringInfo,256,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00920ac0` | `HashList_CodeStringInfo_Slot1Thunk` | ✅ | virtual | Adjustor thunk: adds 0x414 to ECX and jumps to 00785d00. |
| `00920ad0` | `HashList_CodeStringInfo_Slot2Thunk` | ✅ | virtual | Adjustor thunk: adds 0x414 to ECX and jumps to 00785d70. |
| `00925100` | `HashList_CodeStringInfo_Ctor` | ✅ | ctor | Constructor of the HashList base: calls 00921230 on this (HashListMap_CodeStringInfo_Ctor, ECX = this), then stores the HashList<CodeStringMappings::StringInfo,256,0> vtable at this and returns this. |
| `00925320` | `HashList_CodeStringInfo_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 009209e0 on this (bytes: call after the SEH frame setup), then frees the object through _free when bit 0 of the second argument is set; returns this. |
