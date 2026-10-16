# Class: SoeUtil::HashListMap<int,GameCommerce::CreditCardPaymentSource,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d58f40` | `HashListPoolCallSlot1_68 ?` | 🟡 | virtual | Calls FUN_006fb9c0, then FUN_006fb860 to obtain a pool object; if non-null calls its vtable slot 1, else if the global DAT_01bdf280 is non-null calls its vtable slot 1, else calls FUN_006fb950(0x68,4). Same shape as Has… |
| `00d58f80` | `HashListFreeBlock4 ? → SoeUtil::HashListMap<int,GameCommerce::CreditCardPayment…` | 🟡 | virtual | Calls FreeMemory (FUN_00401c00) with the argument and alignment 4 and returns. |
| `00d598e0` | `HashListDtor` | ✅ | dtor | Reads the list head at +8, installs the HashListMap<int,CreditCardPaymentSource,8,-1> vtable in *param_1, and removes each list head through FUN_00d59650 until the list is empty; no storage release. |
| `00d59960` | `HashListScalarDeletingDtor → SoeUtil::HashListMap<int,GameCommerce::CreditCardP…` | ✅ | dtor | Sets the HashListMap<int,CreditCardPaymentSource,8,-1> vtable, removes each list head through FUN_00d59650, and calls _free(this) when bit 0 of the flags argument is set; returns this otherwise. |
