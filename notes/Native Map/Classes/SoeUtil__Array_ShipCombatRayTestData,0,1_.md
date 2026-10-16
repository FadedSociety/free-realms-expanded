# Class: SoeUtil::Array<ShipCombatRayTestData,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53980` | `— ⚠reject` | ⚪ | virtual | Virtual allocate hook (slot 1) of Array<ShipCombatRayTestData,0,1>: reads the capacity at +0xc and clumps the request (request*5/4 when growing, existing buffer returned when capacity >= request*4/3 or on an exact match… |
| `00a53a20` | `SoeUtil::Array_RayTestData_Deallocate` | ✅ | virtual | Virtual deallocate hook (slot 2) of Array<ShipCombatRayTestData,0,1>: forwards the pointer argument to FUN_00401c00 with alignment 0x10 (bytes 8b 44 24 04 6a 10 50 e8 ... c2 04 00). |
| `00a53a40` | `SoeUtil::Array_RayTestData_Swap ⚠downgrade` | 🟢 | virtual | Virtual swap hook (slot 3) of Array<ShipCombatRayTestData,0,1>: exchanges the buffer pointer at +4 and, when unequal, XOR-exchanges count at +8 and capacity at +0xc with the argument object's fields; a non-zero flag tai… |
| `00a55c10` | `SoeUtil::Array_RayTestData_Dtor` | ✅ | dtor | Installs the ray-test array vtable, clears count, frees the allocated buffer with alignment 0x10, and resets the pointer. |
| `00a56510` | `SoeUtil::Array_RayTestData_ScalarDeletingDtor` | ✅ | dtor | Resets ray-test array vtable and count, frees element storage with alignment 0x10, then conditionally frees the containing object. |
