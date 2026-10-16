# Class: SoeUtil::Array<Uram::RedeemCodeResponseOption_const*,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68fd0` | `RedeemCodeOptionArray::Allocate` | ✅ | virtual | When mode is zero, sets target capacity to signed integer-truncated (5*requested)/4 if current capacity is short; if capacity is sufficient and at most (4*requested)/3 it returns the existing pointer and capacity, other… |
| `00d69070` | `RedeemCodeOptionArray::Free ?` | 🟡 | virtual | Frees the supplied array pointer with alignment four. |
| `00d69090` | `RedeemCodeOptionArray::Swap ?` | 🟡 | virtual | In callback mode dispatches through the other object's slot 3; otherwise swaps backing storage, count, and capacity fields. |
| `00d6a220` | `DestroyArrayStorage` | ✅ | dtor | Stores the array vtable, clears its size, frees the backing pointer with alignment 4 through FUN_00401c00, and clears that pointer. |
| `00d6a250` | `ResetArrayStorageAndFreeWhenFlagSet → SoeUtil::Array<Uram::RedeemCodeResponseOp…` | ✅ | virtual | Writes the array vtable, clears size, frees and clears its backing pointer, then frees this only when flag bit 0 is set; it does not call a separate destructor body. |
