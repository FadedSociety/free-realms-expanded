# Class: SoeUtil::Array<ShipCombatZonePickupLocationConfig::PickupInstance,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a52db0` | `SoeUtil::Array_PickupInstance_Deallocate` | ✅ | virtual | Virtual Deallocate hook (slot 2) of the Array<PickupInstance,0,1> specialization: forwards the pointer to FreeMemory with alignment 4. |
| `00a5dda0` | `PickupInstanceArrayDtor` | ✅ | dtor | Replaces the vtable with the array vtable, destroys the array elements, frees the backing buffer with alignment 4, and clears its buffer pointer. |
| `00a5ddf0` | `ArrayResizeAllocate` | ✅ | virtual | Reads current capacity from +0x0c. With flag clear, if capacity < request it allocates floor(5*request/4); otherwise it reuses the current buffer when capacity <= floor(4*request/3), and allocates floor(6*request/5) whe… |
| `00a5de90` | `ArraySwapStorage` | ✅ | virtual | When the flag is clear, swaps the backing-buffer pointer and the two capacity/count fields with the supplied array; when set, calls the supplied array's virtual slot +0xc. |
| `00a5e740` | `PickupInstanceArray::ScalarDeletingDtor` | ✅ | dtor | Resets the array vtable, destroys elements, frees the aligned backing buffer, clears the pointer, and frees the object itself only when delete flag bit 0 is set. |
| `00a65050` | `PickupInstanceArray_Resize` | 🟢 | method | Resize of the pickup-instance array: asks the array's virtual allocate slot (+4) for storage sized by the element count and alignment arguments, and when the storage moved it copy-constructs the surviving 0x40-byte elem… |
