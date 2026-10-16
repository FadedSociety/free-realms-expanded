# Class: InGamePurchaseSessionStats<GameCommerce::ClientInGamePurchaseOrder>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b40370` | `InGamePurchaseSessionStatsCtor` | ✅ | ctor | Constructor of the session-stats object: writes Loggable::vftable then InGamePurchaseSessionStats::vftable at +0, zeroes +4/+5 and +0xc, sets +8 to -1, initialises the two time pairs at +0x10/+0x18 from DAT_01be9680/+68… |
| `00b40630` | `InGamePurchaseSessionStatsDump` | ✅ | virtual | Virtual slot 0 of the session-stats vtable: writes the session summary through FUN_00720c60 - total shopping time (+0x18-+0x10) with referrer id (+8) and referrer context (+0xc) - then for each section prints its counte… |
| `00b40930` | `InGamePurchaseSessionStatsDtor` | ✅ | dtor | Destructor of the session-stats object: rewrites the class vtable, tears down the trailing member through FUN_00b358c0, runs the transaction-tracker list destructor body FUN_00b40250 four times, then the member destruct… |
| `00b40b30` | `InGamePurchaseSessionStats_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 1): runs the session-stats destructor body 00b40930 and, when bit 0 of the flag byte argument is set, releases this with _free; otherwise returns this. |
