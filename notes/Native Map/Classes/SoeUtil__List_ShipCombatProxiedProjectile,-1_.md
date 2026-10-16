# Class: SoeUtil::List<ShipCombatProxiedProjectile,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a52fe0` | `SoeUtil::List_ShipCombatProxiedProjectile_Allocate` | ✅ | virtual | Virtual Allocate hook (slot 2) of List<ShipCombatProxiedProjectile,-1>: uses the thread/global block allocator when present, else allocates a node through FUN_006fb950 with element size 0xa0 and alignment 0x10. |
| `00a53020` | `SoeUtil::List_ShipCombatProxiedProjectile_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 3) of List<ShipCombatProxiedProjectile,-1>: forwards the node pointer to FreeMemory with alignment 0x10. |
| `00a59ab0` | `ShipCombatProxiedProjectileListDtor` | ✅ | dtor | Installs the list vtable and repeatedly removes the current head node until the list is empty. |
| `00a59c70` | `ShipCombatProxiedProjectileListScalarDtor` | ✅ | dtor | Installs the list vtable, removes nodes until empty, then frees the list object when flag bit 0 is set. |
