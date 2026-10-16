# Class: VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerby

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009edde0` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerby_ctor` | ✅ | ctor | Constructor: runs the derby base constructor FUN_009e9f80 with sub-opcode 10, stores the CreateProxiedVehicleDemolitionDerby vtable, initialises the embedded IString member at +0x18 (vtable, static empty buffer DAT_01be… |
| `009ede60` | `— ⚠reject` | ⚪ | dtor | Destructor body: calls FUN_01056260, stores the IString vtable at +0x18 and releases the member's buffer (atomic decrement in front of the pointer at +0x1c; free through IString vtable slot 2 when it hits zero), then re… |
| `009ee200` | `VehicleDemolitionDerbyPacketCreateProxiedVehicleDemolitionDerby_scalarDeletingD…` | ✅ | dtor | Scalar deleting destructor: calls the destructor body 009ede60 (releases the embedded IString member and restores the base vtables) and frees this through _free when bit 0 of param_2 is set; returns this otherwise. |
