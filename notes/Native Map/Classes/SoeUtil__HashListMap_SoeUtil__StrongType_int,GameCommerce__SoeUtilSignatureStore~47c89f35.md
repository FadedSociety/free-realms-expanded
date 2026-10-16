# Class: SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUtilSignatureStoreBundleId,1>,GameCommerce::StoreBundleGroupDefinition::Entry,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d58440` | `HashListPoolCallSlot1_1c ?` | 🟡 | virtual | Calls FUN_006fb9c0, then FUN_006fb860 to obtain a pool object; if non-null calls its vtable slot 1, else if the global DAT_01bdf280 is non-null calls its vtable slot 1, else calls FUN_006fb950(0x1c,4). Same shape as Lis… |
| `00d58480` | `HashListFreeBlock4 ?` | 🟡 | virtual | Calls FreeMemory (FUN_00401c00) with the argument and alignment 4, then returns with ret 4-free stack cleanup as emitted by the compiler. |
| `00d58990` | `HashListDtor → SoeUtil::HashListMap<SoeUtil::StrongType<int,GameCommerce::SoeUt…` | ✅ | dtor | Sets the HashListMap vtable in *param_1 and removes every node from its list (head at +8) by calling FUN_00b290e0 until the head is null; no storage release. |
| `00d58ab0` | `HashListScalarDeletingDtor → SoeUtil::HashListMap<SoeUtil::StrongType<int,GameC…` | ✅ | dtor | Sets the HashListMap vtable, removes every list node with FUN_00b290e0, and calls _free(this) when bit 0 of the flags argument is set; returns this otherwise. |
