# Class: BaseVehicleDemolitionDerbyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009e9f20` | `BaseVehicleDemolitionDerbyPacket_ctorZero` | ✅ | ctor | Constructor: stores the BasePacket vtable, writes the derby base opcode 0x22 at +4, overwrites the vtable with BaseVehicleDemolitionDerbyPacket::vftable and stores 0 as the sub-opcode at +8. |
| `009e9f80` | `BaseVehicleDemolitionDerbyPacket_ctor` | ✅ | ctor | Constructor: stores the BasePacket vtable, writes the base opcode 0x22 at +4, overwrites the vtable with BaseVehicleDemolitionDerbyPacket::vftable and stores the caller-provided sub-opcode at +8. |
| `009e9ff0` | `BaseVehicleDemolitionDerbyPacket_dtor` | ✅ | dtor | Destructor body: rewrites the vtable to BaseVehicleDemolitionDerbyPacket::vftable and then to BasePacket::vftable, with no member teardown. |
| `009ea040` | `BaseVehicleDemolitionDerbyPacket_scalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: restores the BaseVehicleDemolitionDerbyPacket and BasePacket vtables and frees this through _free when bit 0 of the flag byte is set. |
| `009ec880` | `BaseVehicleDemolitionDerbyPacket_CopyCtor` | 🟢 | ctor | Copy-constructor body of the derby packet base: writes BasePacket::vftable, copies the source dword at +4 (the base opcode field), then writes BaseVehicleDemolitionDerbyPacket::vftable and copies the source dword at +8 … |
