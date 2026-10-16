# Class: VehicleLoadoutListDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c2c080` | `VehicleLoadoutListDataSource::GetColumnName` | ✅ | virtual | Virtual column-name accessor (vtable slot [1]): returns LoadoutId (0), Name (1), Description (2), ImageSetId (3), ImageTintValue (4), Quantity (5), MembersOnly (6) and Active (7), with <error> otherwise. |
| `00c2c1d0` | `VehicleLoadoutListDataSource::RebuildRows` | 🟢 | method | Rebuilds the data source row list: notifies through vtable slot [3] (+0xc) of this+0x34, takes the client object at *(this+0x254)+0x3e0, walks its vehicle/inventory entry list from +0x3547c (next at entry+0x20c) and for… |
| `00c2c270` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [4]) with signature (row, column, out): lazily rebuilds the rows via 00c2c1d0 when the valid byte at +0x228 is 0, fetches the row through vtable slot [4] of this+4, and then switches on the c… |
| `00c2c4c0` | `— ⚠reject` | ⚪ | virtual | Virtual method (vtable slot [2]): when the valid byte at +0x228 is zero it rebuilds the rows through 00c2c1d0 and then always calls the shared data-source refresh helper 010c8f30. |
| `00c2c4e0` | `VehicleLoadoutListDataSource::~VehicleLoadoutListDataSource` | 🟢 | dtor | Destructor body: resets the three vtable pointers and calls the shared base teardown 0091bf60, then returns. |
| `00c2c560` | `VehicleLoadoutListDataSource::VehicleLoadoutListDataSource` | ✅ | ctor | Constructor: registers the data-source name 'BaseClient.Vehicle.LoadoutList' through 00922890, installs the three vtable pointers, stores the argument at +0x254, zeroes the valid byte at +0x258 and the id filter at +0x2… |
| `00c2c5e0` | `VehicleLoadoutListDataSource::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the destructor body 00c2c4e0 and frees the object with _free when the low bit of the flag argument is set. |
