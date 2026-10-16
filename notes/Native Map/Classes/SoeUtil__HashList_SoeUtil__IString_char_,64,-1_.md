# Class: SoeUtil::HashList<SoeUtil::IString<char>,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00979080` | `SoeUtil::HashListIStringCtor` | ✅ | ctor | Stores the HashListMap vtable at this+0, zeroes the count at +0x10, head at +8 and tail at +0xc, memsets the 0x100-byte bucket table at this+0x14, zeroes +4, then stores the HashList<IString<char>,64,-1> vtable at this+… |
| `00979100` | `SoeUtil::HashListIStringScalarDeletingDtor` | ✅ | dtor | Calls the base destructor body 00978f10 on this, then frees the object with _free when bit 0 of the flag argument is set; returns this. |
