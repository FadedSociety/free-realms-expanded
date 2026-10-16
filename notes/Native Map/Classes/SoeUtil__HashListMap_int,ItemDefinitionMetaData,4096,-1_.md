# Class: SoeUtil::HashListMap<int,ItemDefinitionMetaData,4096,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c70590` | `SoeUtil_HashListMapItemDef4096::VirtualSlot1 ?` | 🟡 | virtual | Calls 006fb9c0, then gets an object from 006fb860. If non-null it calls that object's vtable slot 1 (+4). Else, if global DAT_01bdf280 is non-null, it calls the same slot on it. Else it calls 006fb950(0x20, 4) and retur… |
| `00c705d0` | `SoeUtil_HashListMapItemDef4096::VirtualSlot2Free4 ?` | 🟡 | virtual | Wrapper: pushes the stack argument and constant 4, calls 00401c00 (free routine with alignment), pops 8 and returns with ret 4. Bytes 8b 44 24 04 6a 04 50 e8 ... c2 04 00. |
| `00c71750` | `HashListMap4096BaseDtor → SoeUtil::HashListMap<int,ItemDefinitionMetaData,4096,…` | ✅ | dtor | Stores the 4096-bucket HashListMap vtable, then calls 00c70520 on the head at +8 until the head is null. No free. |
| `00c717a0` | `HashListMap4096::ScalarDeletingDtor` | ✅ | dtor | Stores the 4096-bucket HashListMap vtable, drains the head at +8 through 00c70520, then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
