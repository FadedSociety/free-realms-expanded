# Class: EquipScreenCurrentlyEquippedDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2c750` | `EquipScreenCurrentlyEquippedDataSource::EquipScreenCurrentlyEquippedDataSource` | ✅ | ctor | Constructor with two arguments: calls the shared inventory-backed base constructor 00cb1bb0('BaseClient.Inventory.CurrentlyEquipped',arg3), installs the four vtable pointers (this+0, +0x30, +0x34, +0x254) and stores arg… |
| `00c2c800` | `EquipScreenCurrentlyEquippedDataSource::~EquipScreenCurrentlyEquippedDataSource` | 🟢 | dtor | Destructor body: resets the four vtable pointers to EquipScreenCurrentlyEquippedDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2c860` | `EquipScreenCurrentlyEquippedDataSource::SetRowId` | 🟢 | method | Setter: stores the argument at this+0x2e0 and calls virtual slot [6] (offset +0x18) of the object at this+0x254 to notify the data source. Its only caller is 009905c0, the same UI path that drives the sibling slot filte… |
| `00c2c880` | `EquipScreenCurrentlyEquippedDataSource::ClearDirty ? ⚠downgrade` | 🟡 | virtual | Virtual method (vtable slot [14]): writes a single zero byte to this+0x2d0 and returns. Decoded bytes are mov byte [ecx+0x2d0],0; ret. |
| `00c2c890` | `EquipScreenCurrentlyEquippedDataSource::AdjustorDtorThunk` | ✅ | dtor | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c8d0 (decoded bytes: sub ecx,0x254; jmp +0x35); the decompiled body is just the call to 00c2c8d0. The binary att… |
| `00c2c8d0` | `EquipScreenCurrentlyEquippedDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable 0183026c slot [0]): calls the destructor body 00c2c800 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2c9c0` | `EquipScreenCurrentlyEquippedDataSource::GetCellText` | 🟢 | virtual | Virtual method (vtable slot [4]) with signature (row, column, out): looks the row up in the client item map (00769fe0 with the key at entry+4), requires the currently-equipped reference at this+0x2b0 and the item manage… |
