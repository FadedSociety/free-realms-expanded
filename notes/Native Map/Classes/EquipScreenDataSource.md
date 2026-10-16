# Class: EquipScreenDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2c620` | `EquipScreenDataSource::EquipScreenDataSource` | ✅ | ctor | Constructor with five arguments: calls the shared inventory-backed base constructor 00cb1bb0(arg4,arg3), installs the four vtable pointers (this+0, +0x30, +0x34, +0x254), stores arg2 at +0x2dc, arg5 at +0x2e0 and zeroes… |
| `00c2c6d0` | `EquipScreenDataSource::~EquipScreenDataSource` | 🟢 | dtor | Destructor body: resets the four vtable pointers (this+0, +0x30, +0x34, +0x254) to EquipScreenDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2c730` | `EquipScreenDataSource::SetSlotFilter ? ⚠rename` | 🟡 | method | Setter: stores the argument at this+0x2e4 and calls virtual slot [6] (offset +0x18) of the object at this+0x254 to notify the data source. Its only caller is 009905c0. |
| `00c2c8a0` | `EquipScreenDataSource::AdjustorDtorThunk` | ✅ | dtor | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c8b0 (decoded bytes: sub ecx,0x254; jmp +0x5); the decompiled body is just the call to 00c2c8b0. The binary attr… |
| `00c2c8b0` | `EquipScreenDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable 018301c4 slot [0]): calls the destructor body 00c2c6d0 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2c8f0` | `— ⚠reject` | ⚪ | virtual | Virtual predicate (vtable slot [15]) with signature (row record): checks that the client's current item exists and its dword at +0x30 equals the slot id at this+0x2e0, then queries the equip state through 00b0d810/00bae… |
| `00c2c980` | `EquipScreenDataSource::GetSpecialColumnText` | 🟢 | virtual | Virtual cell getter (vtable slot [16]) with signature (row record, column, out): for column 4 it formats the integer (record+0x14 - record+0x1c - record+0x18) through 0082ecd0 with the integer format literal 0x017703e0 … |
