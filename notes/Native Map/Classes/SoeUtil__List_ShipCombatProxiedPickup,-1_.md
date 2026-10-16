# Class: SoeUtil::List<ShipCombatProxiedPickup,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53050` | `SoeUtil::List_ShipCombatProxiedPickup_Allocate ⚠downgrade` | 🟢 | virtual | Virtual Allocate hook (slot 2) of List<ShipCombatProxiedPickup,-1>: uses the thread/global block allocator when present, else allocates a node through FUN_006fb950 with element size 0x44 and alignment 4. |
| `00a53090` | `SoeUtil::List_ShipCombatProxiedPickup_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 3) of List<ShipCombatProxiedPickup,-1>: forwards the node pointer to FreeMemory with alignment 4. |
| `00a59ae0` | `ShipCombatProxiedPickupListDtor` | ✅ | dtor | Installs the list vtable and repeatedly removes the current head node until the list is empty. |
| `00a59cb0` | `ShipCombatProxiedPickupListScalarDtor` | ✅ | dtor | Installs the list vtable, removes nodes until empty, then frees the list object when flag bit 0 is set. |
