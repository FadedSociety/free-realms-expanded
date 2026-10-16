# Class: SoeUtil::Array<VehicleEquipmentEntry,16,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bc1d0` | `FreeHeapStorage` | ✅ | virtual | Frees param_2 at alignment 4 only when it differs from the inline address obtained by rounding this+0x13 down to a four-byte boundary. |
| `0090e940` | `VehicleEquipmentEntryArray::VehicleEquipmentEntryArray ?` | 🟡 | ctor | Initializes the four array storage fields to zero and installs the VehicleEquipmentEntry array vtable. |
| `00910430` | `VehicleEquipmentEntryArray::ScalarDeletingDtor ?` | 🟡 | dtor | Calls 00907d60 to destroy the array, frees the object when flag bit 0 is set, and returns its pointer. |
| `00919600` | `SoeUtil::ArrayVehicleEquipmentEntryVirt3 ?` | 🟡 | virtual | Virtual slot [3] of Array<VehicleEquipmentEntry,16,0>. Reads the element pointer and count of the argument object at +4 and +8, calls FUN_00909d50 on each pair, releases the buffer with FUN_008e4580, does the same for t… |
