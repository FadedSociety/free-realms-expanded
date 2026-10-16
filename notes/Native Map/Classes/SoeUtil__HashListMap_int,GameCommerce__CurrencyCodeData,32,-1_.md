# Class: SoeUtil::HashListMap<int,GameCommerce::CurrencyCodeData,32,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b208c0` | `CtorHashListMapCurrencyCode32` | ✅ | ctor | Constructor: stores SoeUtil::HashListMap<int,GameCommerce::CurrencyCodeData,32,-1>::vftable at +0, zeroes +0x10, +8 and +0xc, memsets 0x80 bytes at +0x14 (32 bucket heads) to zero and zeroes +4; returns the object. |
| `00b25070` | `— ⚠reject` | ⚪ | virtual | Calls FUN_006fb9c0, then FUN_006fb860. If the thread-local manager is non-null, invokes its vtable slot +4 and returns; otherwise, if DAT_01bdf280 is non-null, invokes that manager slot +4 and returns; otherwise calls F… |
| `00b250b0` | `SoeUtil::HashListMap_CurrencyCodeDataFreeSlot` | ✅ | virtual | Virtual free slot (vtable index 2): FreeMemory(param_1, 4) on the node pointer. |
| `00b40a30` | `HashListMapCurrencyCodeData_Destruct` | ✅ | dtor | Destructor body of the currency-code map: writes the class vtable at +0 and drains the element list at +8 through FUN_00b3a930 on each head until the head is null. |
| `00b40b50` | `HashListMapCurrencyCodeData_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): rewrites the vtable, drains the element list at +8 through FUN_00b3a930, and when bit 0 of the flag byte argument is set releases this with _free; otherwise returns this. |
