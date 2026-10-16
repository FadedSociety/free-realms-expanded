# Class: SoeUtil::HashListMap<int,ItemDefinitionMetaData,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c70660` | `SoeUtil_HashListMapItemDef64::VirtualSlot1 ?` | 🟡 | virtual | Same routing as 00c70590: calls 006fb9c0, then the object from 006fb860 (slot 1 call when non-null), else the global DAT_01bdf280, else calls 006fb950(0x20, 4) and returns. |
| `00c706a0` | `SoeUtil_HashListMapItemDef64::VirtualSlot2Free4 ?` | 🟡 | virtual | Wrapper: pushes the stack argument and constant 4, calls 00401c00 (free routine with alignment), pops 8 and returns with ret 4. |
| `00c71c00` | `HashListMap64BaseDtor → SoeUtil::HashListMap<int,ItemDefinitionMetaData,64,-1>:…` | ✅ | dtor | Stores the 64-bucket HashListMap vtable, then calls 00c70910 on the head at +8 until the head is null. No free. |
| `00c71cd0` | `HashListMap64::ScalarDeletingDtor` | ✅ | dtor | Stores the 64-bucket HashListMap vtable, drains the head at +8 through 00c70910, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
