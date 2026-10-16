# Class: SoeUtil::HashMap<int,ClientInGamePurchaseManager::CreditCardTypeHashEntry,24,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b207a0` | `CtorHashMapCreditCardType24` | ✅ | ctor | Constructor: stores SoeUtil::HashMap<int,ClientInGamePurchaseManager::CreditCardTypeHashEntry,24,-1>::vftable at +0, zeroes the dword at +8, memsets 0x60 bytes at +0xc (24 bucket heads, 24*4) to zero and zeroes +4; retu… |
| `00b24dd0` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b24e10` | `SoeUtil::HashMap_CreditCardTypeHashEntryFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the node pointer. |
| `00b39990` | `CreditCardTypeHashMap::Dtor` | ✅ | dtor | Installs typed 24-bucket map vtable and jumps into shared bucket-clear routine. |
