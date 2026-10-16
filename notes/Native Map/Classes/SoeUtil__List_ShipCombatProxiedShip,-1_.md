# Class: SoeUtil::List<ShipCombatProxiedShip,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ca4260` | `— ⚠reject` | ⚪ | virtual | Obtains the current allocator, returns its interface when present, otherwise returns the fallback allocator or invokes the allocation failure path. |
| `00ca42a0` | `List_FreeNode → SoeUtil::List<ShipCombatProxiedShip,-1>::FreeNode` | ✅ | virtual | Frees the supplied node storage through the shared memory helper using alignment 0x10. |
| `00ca4c00` | `List_Dtor → SoeUtil::List<ShipCombatProxiedShip,-1>::Dtor` | ✅ | dtor | Restores the list vtable and repeatedly unlinks and destroys the current head node until the list is empty. |
| `00ca4c60` | `List_ScalarDeletingDtor → SoeUtil::List<ShipCombatProxiedShip,-1>::ScalarDeleti…` | ✅ | dtor | Restores the list vtable, drains and destroys all nodes, and frees the list allocation when the deleting flag is set. |
