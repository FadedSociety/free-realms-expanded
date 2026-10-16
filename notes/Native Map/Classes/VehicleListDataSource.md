# Class: VehicleListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2bf00` | `VehicleListDataSource::VehicleListDataSource` | ✅ | ctor | Constructor: calls 00cb1bb0('BaseClient.VehicleList', arg2) - the shared inventory-backed data-source base constructor - then installs the four vtable pointers (this+0, +0x30, +0x34, +0x254) and zeroes the byte/dword me… |
| `00c2bfa0` | `VehicleListDataSource::~VehicleListDataSource` | 🟢 | dtor | Destructor body: resets the four vtable pointers (this+0, +0x30, +0x34, +0x254) to VehicleListDataSource::vftable and calls the shared base teardown 00cb1b10, then returns. |
| `00c2c000` | `VehicleListDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): for column 6 it returns the literal 'Active Vehicle' and for every other column index it forwards to the shared inventory-item column naming helper 00cb09d0, whose context… |
| `00c2c020` | `VehicleListDataSource::SetActiveVehicleId` | 🟢 | method | Setter: when the argument differs from the dword at this+0x2dc it stores it and then calls virtual slot [6] (offset +0x18) of the object at this+0x254, which notifies the data source that its Active Vehicle column chang… |
| `00c2c050` | `VehicleListDataSource::IsActiveVehicleRow` | 🟢 | virtual | Virtual predicate (vtable slot [15]): resolves an object with 00b0d810 and returns 1 when it exists and its dword at +0x70 equals the active-vehicle id stored at this+0x2dc, otherwise 0. |
| `00c2c110` | `VehicleListDataSource::AdjustorDtorThunk` | ✅ | dtor | Adjustor thunk for the secondary vtable: subtracts 0x254 from ECX and jumps to the deleting destructor 00c2c130 (decoded bytes: sub ecx,0x254; jmp +0x15); the decompiled body is just the call to 00c2c130. The binary att… |
| `00c2c130` | `VehicleListDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable 01830024 slot [0]): calls the destructor body 00c2bfa0, tests the low bit of the flag argument and frees the object with _free when set (decoded bytes: push esi; mov esi,ecx; call 00c2… |
| `00c2c160` | `VehicleListDataSource::GetSpecialCellText` | 🟢 | virtual | Virtual cell getter (vtable slot [16]) with signature (row, column, out): for column 6 (Active Vehicle) it formats 1 or 0 through 00703620 with the true/false literals at 0x0180c498 / 0x017f7d9c after comparing the row'… |
