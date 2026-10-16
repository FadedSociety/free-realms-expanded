# Class: SoeUtil::HashListMap<int,SoeUtil::IString<char>,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa6570` | `HashListMapCtor_IStringInt16` | ✅ | ctor | Stores SoeUtil::HashListMap<int,SoeUtil::IString<char>,16,-1>::vftable, zeroes the head/tail/count fields and clears the 0x40-byte bucket array at +0x14 with _memset. |
| `00ab9c50` | `SoeUtil_HashListMap_IString_16_Dtor` | ✅ | dtor | Destructor body: stamps the HashListMap<int,IString<char>,16,-1> vftable 01818acc at +0 and drains the node list through FUN_00ab32f0 while the counter at +8 is nonzero. |
| `00aba380` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (vtable slot 0 of the IString-valued 16-bucket map): stores vftable 01818acc, drains the node list through FUN_00ab32f0 while the count at +8 is nonzero, then calls _free on this when bit 0 of… |
| `00abaad0` | `SoeUtil_HashListMap_IString_16_Dtor2` | ✅ | dtor | Destructor body: reads the count at +8, stamps the HashListMap<int,IString<char>,16,-1> vftable 01818acc at +0, and while the count is nonzero calls FUN_00ab32f0 on the list and reloads the count. |
| `00abc200` | `SoeUtil_HashListMap_IString_16_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,IString<char>,16,-1> vftable 01818acc, zeroes the count at +8 and the sixteen bucket heads (0x40 bytes from +0x14, plus +0xc and +0x10), then stamps the HashList<IString<char>,16,… |
