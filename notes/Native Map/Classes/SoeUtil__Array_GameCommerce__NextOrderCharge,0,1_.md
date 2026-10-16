# Class: SoeUtil::Array<GameCommerce::NextOrderCharge,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68d10` | `NextOrderChargeArray::Allocate` | ✅ | virtual | When mode is zero, sets target capacity to signed integer-truncated (5*requested)/4 if current capacity is short; if capacity is sufficient and at most (4*requested)/3 it returns the existing pointer and capacity, other… |
| `00d68db0` | `NextOrderChargeArray::Free ?` | 🟡 | virtual | Frees the supplied array pointer through FreeMemory with alignment four. |
| `00d68dd0` | `NextOrderChargeArray::Swap ?` | 🟡 | virtual | For nonzero mode calls a callback on the other array; otherwise swaps storage pointer, count, and capacity fields between arrays. |
