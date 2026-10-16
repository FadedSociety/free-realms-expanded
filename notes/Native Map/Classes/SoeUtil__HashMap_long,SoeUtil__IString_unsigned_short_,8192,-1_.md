# Class: SoeUtil::HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be7ec0` | `HashMapLongIString16_8192_Ctor` | ✅ | ctor | Stores the SoeUtil::HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1> vtable at *this, zeroes the count at +4 and the 0x8000-byte bucket area at +0xc through _memset, zeroes the dword at +8 and returns this. |
| `00be7fa0` | `HashMapLongIString16_Slot1 ?` | 🟡 | virtual | Calls 006fb9c0, then calls the object returned by 006fb860 through its vtable slot +4 if non-null; otherwise calls the same slot on the global object DAT_01bdf280 if set; otherwise calls 006fb950(0x18, 4). |
| `00be7fe0` | `HashMapLongIString16_FreeArg4 ?` | 🟡 | virtual | Takes one stack argument and calls 00401c00(argument, 4) (FreeMemory with alignment 4), then returns with ret 4; it is a plain free of its argument, with no destructor body. |
| `00be8680` | `HashMapLongIString16_DtorBody` | ✅ | dtor | Stores the HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1> vtable at *this and calls 00be8270 (bucket clear) on this; returns void. |
| `00be8720` | `HashMapLongIString16_ScalarDtor` | 🟢 | dtor | Stores the HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1> vtable at *this, calls 00be8270 (bucket clear), then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00be8750` | `HashMapLongIString16_CtorLeadArg` | ✅ | ctor | Stores its dword argument at *this, stores the SoeUtil::HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1> vtable at this+4, zeroes this+8 and this+0xc, zeroes 0x8000 bytes at this+0x10 through _memset and returns t… |
| `00be87d0` | `— ⚠reject` | ⚪ | dtor | Calls 00be8270 (bucket clear) on this, stores the HashMap<long,SoeUtil::IString<unsigned_short>,8192,-1> vtable at this+4, then calls 00be8270 again; returns void. |
