# Class: VehicleEquipScreenCurrentlyEquippedDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2cc00` | `VehicleEquipScreenCurrentlyEquippedDataSource::VehicleEquipScreenCurrentlyEquip…` | ✅ | ctor | Constructor with two arguments: calls the shared inventory-backed base constructor 00cb1bb0('BaseClient.Vehicle.CurrentlyEquipped',arg3), installs the four vtable pointers and stores arg2 at +0x2dc, returning this. |
| `00c2ccb0` | `VehicleEquipScreenCurrentlyEquippedDataSource::~VehicleEquipScreenCurrentlyEqui…` | 🟢 | dtor | Destructor body: resets the four vtable pointers to VehicleEquipScreenCurrentlyEquippedDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2cd30` | `VehicleEquipScreenCurrentlyEquippedDataSource::ClearDirty ? ⚠downgrade` | 🟡 | virtual | Virtual method (vtable slot [14]): writes a single zero byte to this+0x2d0 and returns. Decoded bytes are mov byte [ecx+0x2d0],0; ret. |
| `00c2cd50` | `VehicleEquipScreenCurrentlyEquippedDataSource::AdjustorDtorThunk` | ✅ | dtor | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2cd80 (decoded bytes: sub ecx,0x254; jmp +0x25); the decompiled body is just the call to 00c2cd80. Attributed by … |
| `00c2cd80` | `VehicleEquipScreenCurrentlyEquippedDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable 018303f4 slot [0]): calls the destructor body 00c2ccb0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2cef0` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [4]) with signature (row, column, out): for rows below 0x10 with the reference at this+0x2b0 and the item manager at *(this+0x2ac)+0x3e0 present, it resolves the vehicle record with 0101d0f0(… |
