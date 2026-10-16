# Class: SoeUtil::Array<MiniGameGoalState,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b5390` | `— ⚠reject` | ⚪ | virtual | For positive requests, chooses capacity from request, current capacity at this+0xc and flag. With flag clear: if current<request, capacity becomes ceil(request*5/4); if current<=floor(request*4/3), returns current stora… |
| `009b5430` | `ArrayMiniGameGoalStateSwap` | ✅ | virtual | If flag is set, invokes vtable slot 3 on the other object; otherwise swaps backing pointer, element count and capacity with the supplied array. |
| `009b5bd0` | `ArrayMiniGameGoalState::ScalarDeletingDtor` | ✅ | virtual | Restores goal-state array vtable, destroys elements, frees backing storage, then optionally frees object when flag bit 0 is set. |
