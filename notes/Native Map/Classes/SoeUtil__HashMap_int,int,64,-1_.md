# Class: SoeUtil::HashMap<int,int,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c70600` | `HashMapIntInt64_VirtualSlot1 ?` | 🟡 | virtual | Same routing as 00c70590: calls 006fb9c0, gets an object from 006fb860 (slot 1 call when non-null), else the same slot on DAT_01bdf280, else calls 006fb950(0xc, 4) and returns. |
| `00c70640` | `HashMapIntInt64_VirtualSlot2Free4 ?` | 🟡 | virtual | Wrapper: pushes the stack argument and constant 4, calls 00401c00 (free routine with alignment), pops 8 and returns with ret 4. |
| `00c70780` | `HashMapIntInt64::ScalarDeletingDtor` | ✅ | dtor | Stores SoeUtil::HashMap<int,int,64,-1>::vftable, calls 00c6f7e0 (node release), then calls _free(this) through thunk 00d82463 when bit 0 of the second argument is set. Returns this. |
| `00c70f90` | `HashMapIntInt64_DtorEH` | ✅ | dtor | Stores SoeUtil::HashMap<int,int,64,-1>::vftable and calls 00c6f7e0 (64-bucket node release) with an EH frame installed. No free; returns nothing. |
| `00c74710` | `— ⚠reject` | ⚪ | ctor | Initialises the HashMap<int,int,64,-1> member at param_1+0xd460 of an enclosing object: stores its vtable, calls FUN_00c6f7e0 and FUN_00a3c480 for the member setup, and uses FUN_00c72f20 and FUN_00cf1530 as cleanup unde… |
