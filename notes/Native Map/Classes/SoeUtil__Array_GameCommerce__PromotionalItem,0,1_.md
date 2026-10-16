# Class: SoeUtil::Array<GameCommerce::PromotionalItem,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68e60` | `PromotionalItemArray::Allocate` | ✅ | virtual | When mode is zero, sets target capacity to signed integer-truncated (5*requested)/4 if current capacity is short; if capacity is sufficient and at most (4*requested)/3 it returns the existing pointer and capacity, other… |
| `00d68f00` | `PromotionalItemArray::Free ?` | 🟡 | virtual | Frees the supplied array storage pointer with alignment four. |
| `00d68f20` | `PromotionalItemArray::Swap ?` | 🟡 | virtual | In nonzero mode dispatches a callback; otherwise swaps the two arrays' storage pointers, element counts, and capacities. |
