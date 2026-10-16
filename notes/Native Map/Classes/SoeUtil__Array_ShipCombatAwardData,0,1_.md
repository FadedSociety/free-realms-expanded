# Class: SoeUtil::Array<ShipCombatAwardData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53850` | `— ⚠reject` | ⚪ | virtual | Virtual allocate hook (slot 1) of Array<ShipCombatAwardData,0,1>: reads the capacity at +0xc and clumps the request (request*5/4 when growing, existing buffer returned when capacity >= request*4/3 or on an exact match u… |
| `00a538f0` | `SoeUtil::Array_AwardData_Deallocate` | ✅ | virtual | Virtual deallocate hook (slot 2) of Array<ShipCombatAwardData,0,1>: forwards the pointer argument to FUN_00401c00 with alignment 8 (bytes 8b 44 24 04 6a 08 50 e8 ... c2 04 00). |
| `00a53910` | `SoeUtil::Array_AwardData_Swap ⚠downgrade` | 🟢 | virtual | Virtual swap hook (slot 3) of Array<ShipCombatAwardData,0,1>: exchanges the buffer pointer at +4 and, when unequal, XOR-exchanges count at +8 and capacity at +0xc with the argument object's fields; a non-zero flag tail-… |
| `00a55b20` | `SoeUtil::Array_AwardData_Dtor` | ✅ | dtor | Installs the ShipCombatAwardData array vtable, clears count, frees the backing array with alignment eight, and clears its pointer. |
| `00a564d0` | `SoeUtil::Array_AwardData_ScalarDeletingDtor` | ✅ | dtor | Resets award-array vtable, clears count, frees element storage with alignment eight, and conditionally frees the object. |
