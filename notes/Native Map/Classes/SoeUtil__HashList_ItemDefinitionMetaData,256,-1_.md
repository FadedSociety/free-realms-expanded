# Class: SoeUtil::HashList<ItemDefinitionMetaData,256,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c70720` | `SoeUtilHashList256_VirtualSlot1 ?` | 🟡 | virtual | Same routing as 00c70590: calls 006fb9c0, gets an object from 006fb860 (slot 1 call when non-null), else the global DAT_01bdf280, else calls 006fb950(0x20, 4) and returns. |
| `00c70760` | `SoeUtilHashList256_VirtualSlot2Free4 ?` | 🟡 | virtual | Wrapper: pushes the stack argument and constant 4, calls 00401c00 (free routine with alignment), pops 8 and returns with ret 4. Bytes 8b 44 24 04 6a 04 50 e8 ... c2 04 00. |
| `00c73c30` | `SoeUtil::HashList_ItemDefinitionMetaData_256_neg1::Ctor` | ✅ | ctor | Constructor: stores the HashListMap<...,256,-1> vtable, then the HashList<...,256,-1> vtable; zeroes this+8, this+0xc, this+0x10 and this+4; memsets 0x400 bytes (256 bucket heads) at this+0x14. Has an SEH frame. |
| `00c73d30` | `SoeUtil::HashList_ItemDefinitionMetaData_256_neg1::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c72280 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
