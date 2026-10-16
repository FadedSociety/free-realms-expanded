# Class: SoeUtil::HashListMap<int,ShipCombatShipItem,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a50780` | `SoeUtil::HashListMap_Int_ShipCombatShipItem_Ctor` | 🟢 | ctor | Constructor of the hash-list map specialization keyed by int with ShipCombatShipItem elements: stores the class vtable, zeroes the element list pointers at +8/+0x10, the count at +0xc and the bucket pointer at +4, and m… |
| `00a52e20` | `SoeUtil::HashListMap_ShipCombatShipItem_Allocate` | ✅ | virtual | Virtual Allocate hook (slot 1) shared by the HashListMap and HashList ShipCombatShipItem specializations: asks the thread's block allocator (FUN_006fb9c0/FUN_006fb860, then the global at DAT_01bdf280) and, when none is … |
| `00a52e60` | `SoeUtil::HashListMap_ShipCombatShipItem_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 2) shared by the HashListMap and HashList ShipCombatShipItem specializations: forwards the pointer to FreeMemory with alignment 4. |
| `00a64a30` | `SoeUtil::HashListMap_ShipItemDtor ⚠downgrade` | 🟢 | dtor | Destructor body of the ship-item hash list: installs SoeUtil::HashListMap<int,ShipCombatShipItem,20,-1>::vftable at +0 and then drains the node list at +8 by repeatedly calling the node remover 00a5d520 until the head i… |
| `00a64c50` | `HashListMapShipItem_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the ship-item hash list: installs SoeUtil::HashListMap<int,ShipCombatShipItem,20,-1>::vftable, drains the +8 node list with 00a5d520 (identical body to 00a64a30) and, when bit 0 of param_2 … |
| `00a64d50` | `HashListMapShipItem_DtorBody` | ✅ | dtor | SEH-protected destructor body of the ship-item hash list: installs SoeUtil::HashListMap<int,ShipCombatShipItem,20,-1>::vftable and drains the +8 node list through 00a5d520 inside an exception frame; the containing objec… |
| `00a674c0` | `HashListMap_ShipCombatShipItem_CopyCtor` | ✅ | ctor | Copy constructor for a 20-bucket hash map keyed by int: stores the HashListMap<int,ShipCombatShipItem,20,-1> vtable, zeroes +4/+8/+0xc/+0x10, memsets the 20 inline buckets at +0x14 (0x50 bytes), rewrites the object vtab… |
