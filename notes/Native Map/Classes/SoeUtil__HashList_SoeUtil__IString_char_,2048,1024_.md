# Class: SoeUtil::HashList<SoeUtil::IString<char>,2048,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091e840` | `HashList_IString_Ctor` | ✅ | ctor | Constructor of the large string hash list: runs the base constructor FUN_00919460 (stack-cookie guarded) and then stores SoeUtil::HashList<SoeUtil::IString<char>,2048,1024>::vftable. |
| `00920b00` | `HashList_IString2048_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 00917e70 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
