# Class: SoeUtil::HashListMap<int,ShipCombatPlayer,0,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a52f00` | `SoeUtil::HashListMap_ShipCombatPlayer_Allocate` | ✅ | virtual | Virtual Allocate hook (slot 1) of the ShipCombatPlayer hash-list map: uses the thread/global block allocator when present, else allocates through FUN_006fb950 with element size 0x368 and alignment 8. |
| `00a52f40` | `SoeUtil::HashListMap_ShipCombatPlayer_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 2) of the ShipCombatPlayer hash-list map: forwards the pointer to FreeMemory with alignment 8. |
| `00a6af80` | `HashListMap_ShipCombatPlayer_Destruct` | ✅ | dtor | Destructor body: stores its own vtable, then drains the node chain by repeatedly calling FUN_00a68c10 on the head at +8 (each call unlinks and releases the node), and finally frees the bucket table pointer stored at +0x… |
| `00a6aff0` | `HashListMap_ShipCombatPlayer_ScalarDeletingDtor` | ✅ | dtor | MSVC scalar deleting destructor in HashListMap<int,ShipCombatPlayer,0,-1> vtable slot 0: calls 00a6af80 on this, tests delete-flag bit 0, conditionally frees this through _free, and returns this when not freed. |
| `00a6b750` | `HashListMap_ShipCombatPlayer_Construct` | ✅ | ctor | Constructor: stores the HashListMap<int,ShipCombatPlayer,0,-1> vtable, clears the fields at +4/+8/+0xc/+0x10 and the bucket pointer at +0x14, then rewrites the object vtable to the HashList<ShipCombatPlayer,0,-1> base v… |
