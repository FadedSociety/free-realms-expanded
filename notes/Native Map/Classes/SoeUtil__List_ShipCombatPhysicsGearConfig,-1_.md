# Class: SoeUtil::List<ShipCombatPhysicsGearConfig,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a64b00` | `PhysicsGearConfigList_Dtor` | ✅ | dtor | Destructor body of the physics gear list: installs SoeUtil::List<ShipCombatPhysicsGearConfig,-1>::vftable and drains the node list at +4 by repeatedly calling the node remover 00a5d210. |
| `00a64b40` | `ListAllocNode0x48` | ✅ | virtual | Allocator slot of the physics gear list vtable: asks the current-thread allocator (006fb9c0 then a virtual +4 call on the object returned by 006fb860) for a 0x48-byte block with alignment 4, falls back to the global all… |
| `00a64b80` | `ListFreeNodeAlign4` | ✅ | virtual | Free slot of the physics gear list vtable: releases param_1 with alignment 4 through 00401c00 (the thread/global memory-manager free) and returns with a 4-byte argument cleanup (ret 4); the raw bytes are 8b 44 24 04 / 6… |
| `00a64d10` | `PhysicsGearConfigList_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the physics gear list: installs SoeUtil::List<ShipCombatPhysicsGearConfig,-1>::vftable, drains the +4 node list with 00a5d210 and, when bit 0 of param_2 is set, releases the list object thr… |
