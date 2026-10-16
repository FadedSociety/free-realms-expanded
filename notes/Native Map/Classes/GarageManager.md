# Class: GarageManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b86d40` | `GarageManager::GarageManager` | ✅ | ctor | Constructor of GarageManager: stores param_2 at +0x10, installs GarageManager::vftable, zeroes +0x14/+0x18/+0x20, sets the flag bytes at +0x1c and +0x24 to 0 and returns. |
| `00b86da0` | `GarageManager::Stop` | 🟢 | method | If the active flag byte at this+0x24 is set, runs the shutdown sequence (FUN_009d2640, FUN_009d2670, FUN_008d2950, FUN_0094c8a0(1)), then obtains the player/HUD object through virtual slot 9 of this+0x10->+200, and when… |
| `00b86ff0` | `GarageManager::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of GarageManager: reinstalls GarageManager::vftable and frees the manager object when bit 0 of param_2 is set. |
| `00b877d0` | `GarageManager::Start` | ✅ | method | Opens the vehicle garage: when the active flag at this+0x24 is clear it constructs and (if this+0x10->+0x280 is set) sends the config request via 00b86a80/00b87310, runs FUN_008c5d90(0x12), calls the player object's vir… |
| `00b87970` | `— ⚠reject` | ⚪ | method | When the garage active flag this+0x24 is set: parses a position/vector (FUN_0101cd40/FUN_0101cd50), calls FUN_008becc0 which must return non-zero, then constructs the equipment-array update packet 00b86b30, stores the r… |
| `00b87a30` | `GarageManager::SelectVehicle` | 🟢 | method | Selects a vehicle: constructs the select-vehicle packet 00b86be0 with the two arguments stored at packet+0xc/+0x10, sends it via 00b87530 when this+0x10->+0x280 is set, calls FUN_00990600(param_3), writes param_3 to pla… |
| `00b87b50` | `GarageManager::RemovePartFromSlot ? ⚠downgrade` | 🟡 | method | When the garage active flag this+0x24 is set: constructs the remove-part packet 00b86c90, stores the slot argument at packet+0xc and sends it through 00b87640 when this+0x10->+0x280 is set; the temporary packet is destr… |
