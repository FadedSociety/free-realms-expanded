# Class: SoeUtil::Array<ShipCombatShipDamageStateConfig,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a53820` | `SoeUtil::Array_ShipDamageStateConfig_Deallocate` | ✅ | virtual | Virtual deallocate hook (slot 2) of Array<ShipCombatShipDamageStateConfig,0,1>: forwards the pointer argument to FUN_00401c00 with alignment 4 (bytes 8b 44 24 04 6a 04 50 e8 ... c2 08 00). |
| `00a61430` | `ShipDamageStateArrayDtor` | ✅ | dtor | Resets the array vtable, destroys the elements, frees the backing storage with alignment 4, and clears the backing pointer. |
| `00a61480` | `ArrayResizeAllocate` | ✅ | virtual | Reads current capacity from +0x0c. With flag clear, if capacity < request it allocates floor(5*request/4); otherwise it reuses the current buffer when capacity <= floor(4*request/3), and allocates floor(6*request/5) whe… |
| `00a61520` | `ArraySwapStorage` | ✅ | virtual | When the flag is clear, swaps the backing-buffer pointer and the two array counters with the supplied array; when set, invokes the supplied object's virtual slot +0xc. |
| `00a616a0` | `ShipDamageStateArray::ScalarDeletingDtor` | ✅ | dtor | Resets the array vtable, destroys its elements, frees the aligned backing buffer, and frees the array object when delete flag bit 0 is set. |
