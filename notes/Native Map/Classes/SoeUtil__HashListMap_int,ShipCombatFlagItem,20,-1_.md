# Class: SoeUtil::HashListMap<int,ShipCombatFlagItem,20,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a507c0` | `SoeUtil::HashListMap_Int_ShipCombatFlagItem_Ctor` | ✅ | ctor | Constructor of the hash-list map specialization keyed by int with ShipCombatFlagItem elements: stores the class vtable, zeroes the element list pointers and count, and memsets the twenty inline bucket entries (0x50 byte… |
| `00a52e90` | `SoeUtil::HashListMap_ShipCombatFlagItem_Allocate` | ✅ | virtual | Virtual Allocate hook (slot 1) of the ShipCombatFlagItem hash-list map: uses the thread/global block allocator when present, else allocates through FUN_006fb950 with element size 0x6c and alignment 4. |
| `00a52ed0` | `SoeUtil::HashListMap_ShipCombatFlagItem_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 2) of the ShipCombatFlagItem hash-list map: forwards the pointer to FreeMemory with alignment 4. |
| `00a64a60` | `SoeUtil::HashListMap_FlagItemDtor` | ✅ | dtor | Destructor body of the ship-flag hash list: installs SoeUtil::HashListMap<int,ShipCombatFlagItem,20,-1>::vftable at +0 and drains the node list at +8 by repeatedly calling the node remover 00a5d640 until the head is nul… |
| `00a64c90` | `HashListMapFlagItem_ScalarDtor` | ✅ | dtor | Scalar deleting destructor of the ship-flag hash list: installs SoeUtil::HashListMap<int,ShipCombatFlagItem,20,-1>::vftable, drains the +8 node list with 00a5d640 and, when bit 0 of param_2 is set, releases the object t… |
| `00a64dc0` | `HashListMapFlagItem_DtorBody` | ✅ | dtor | SEH-protected destructor body of the ship-flag hash list: installs SoeUtil::HashListMap<int,ShipCombatFlagItem,20,-1>::vftable and drains the +8 node list through 00a5d640 inside an exception frame. |
| `00a67560` | `HashListMap_ShipCombatFlagItem_CopyCtor` | ✅ | ctor | Copy constructor for the 20-bucket hash map of ShipCombatFlagItem values: stores the HashListMap<int,ShipCombatFlagItem,20,-1> vtable, zeroes +4/+8/+0xc/+0x10, memsets the 20 inline buckets at +0x14 (0x50 bytes), rewrit… |
