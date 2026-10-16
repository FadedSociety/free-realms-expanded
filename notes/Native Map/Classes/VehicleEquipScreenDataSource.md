# Class: VehicleEquipScreenDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2ca80` | `VehicleEquipScreenDataSource::VehicleEquipScreenDataSource` | ✅ | ctor | Constructor with five arguments: calls the shared inventory-backed base constructor 00cb1bb0(arg4,arg3), installs the four vtable pointers, stores arg2 at +0x2dc, arg5 at +0x2e0 and zeroes +0x2e4, returning this. |
| `00c2cb20` | `VehicleEquipScreenDataSource::~VehicleEquipScreenDataSource` | 🟢 | dtor | Destructor body: resets the four vtable pointers to VehicleEquipScreenDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2cb80` | `— ⚠reject` | ⚪ | virtual | Virtual predicate (vtable slot [15]): requires the client's current item (00b0d810) to match both the slot id at this+0x2e0 (item+0x30) and the vehicle type at this+0x2e8 (item+0x6c), then resolves the vehicle record th… |
| `00c2cd10` | `VehicleEquipScreenDataSource::SetSlotFilter` | 🟢 | method | Setter: stores the argument at this+0x2e0 and calls virtual slot [6] (offset +0x18) of the object at this+0x254 to notify the data source. Its only caller is 00990600. |
| `00c2cd40` | `VehicleEquipScreenDataSource::AdjustorDtorThunk` | ✅ | dtor | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2cd60 (decoded bytes: sub ecx,0x254; jmp +0x15); the decompiled body is just the call to 00c2cd60. Attributed by … |
| `00c2cd60` | `VehicleEquipScreenDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable 0183034c slot [0]): calls the destructor body 00c2cb20 and frees the object with _free when the low bit of the flag argument is set. |
| `00c2cda0` | `VehicleEquipScreenDataSource::GetSpecialColumnText` | 🟢 | virtual | Virtual cell getter (vtable slot [16]) with signature (row record, column, out): for column 0xc it resolves the vehicle record through 0101d0f0(this+0x2e4), scans up to 16 entries of the record's array at record+8 count… |
