# Class: SoeUtil::List<ItemDefinitionMetaData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c70460` | `SoeUtil_ListItemDef::VirtualSlot2 ?` | 🟡 | virtual | Calls 006fb9c0, then gets an object from 006fb860. If that is non-null it calls the object's vtable slot 1 (+4). Otherwise, if global DAT_01bdf280 is non-null, it calls the same slot on that global. Otherwise it calls 0… |
| `00c704a0` | `SoeUtil_ListItemDef::VirtualSlot3Free4 ?` | 🟡 | virtual | Wrapper that pushes the stack argument and the constant 4, calls 00401c00 (free with alignment), and returns with ret 4 after popping 8 bytes. Bytes 8b 44 24 04 6a 04 50 e8 ... c2 04 00. |
| `00c71bb0` | `SoeUtilListItemDef::BaseDtor → SoeUtil::List<ItemDefinitionMetaData,-1>::BaseDt…` | ✅ | dtor | Stores the SoeUtil::List<ItemDefinitionMetaData,-1> vtable into [ecx] and tail-jumps (e9) to 00c716f0 (ListDrainHeadCallSlot3). Eleven bytes: c7 01 e8 3b 83 01 e9 35 fb ff ff. |
| `00c71ca0` | `SoeUtilListItemDef::ScalarDeletingDtor` | ✅ | dtor | Stores the List<ItemDefinitionMetaData,-1> vtable, calls 00c716f0 (list drain), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
