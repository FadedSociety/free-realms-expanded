# Class: VehiclePartPacketRequestRemovePartFromSlot

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b86c90` | `— ⚠reject` | ⚪ | ctor | Constructor of the remove-part-from-slot request packet: runs the base packet constructor FUN_009d2370 with sub-opcode constant 6 and installs the class vtable. |
| `00b86f80` | `VehiclePartPacketRequestRemovePartFromSlot::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the remove-part request packet: resets the base vtables and frees the object through _free when bit 0 of param_2 is set. |
