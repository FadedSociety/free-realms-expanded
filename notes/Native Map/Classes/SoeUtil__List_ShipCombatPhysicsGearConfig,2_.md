# Class: SoeUtil::List<ShipCombatPhysicsGearConfig,2>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a65b10` | `PhysicsGearConfigList2_Dtor` | ✅ | dtor | Destructor body of the pooled physics-gear list variant: installs SoeUtil::List<ShipCombatPhysicsGearConfig,2>::vftable, drains the +4 node list with 00a5d210, releases the pool block tree root at +0x10 (its buffers at … |
| `00a665a0` | `PhysicsGearPool_GetSlotThunk` | ✅ | virtual | Vtable thunk of the pooled list: adjusts the this pointer by +0x10 (the embedded pool object inside SoeUtil::List<ShipCombatPhysicsGearConfig,2>) and tail-jumps to the slot acquisition 00a664c0 - the raw body is 83 c1 1… |
| `00a665b0` | `PhysicsGearPool_PutSlotThunk` | ✅ | virtual | Vtable thunk of the pooled list: adjusts the this pointer by +0x10 (embedded pool object) and tail-jumps to the slot release 00a66530 - the raw body is 83 c1 10 / e9 78 ff ff ff. |
| `00a665c0` | `PhysicsGearPool_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the pooled physics-gear list: runs the destructor body 00a65b10 (which drains the list, releases the pool block tree and restores the non-pooled vtable) and, when bit 0 of param_2 is set, r… |
