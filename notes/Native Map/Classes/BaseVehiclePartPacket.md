# Class: BaseVehiclePartPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d2310` | `BaseVehiclePartPacket::Ctor` | ✅ | ctor | Installs BasePacket, writes packet opcode 0x8b, switches to the BaseVehiclePartPacket vtable, and clears the subtype field. |
| `009d2370` | `BaseVehiclePartPacket::CtorWithSubtype` | ✅ | ctor | Installs BasePacket, writes opcode 0x8b, installs the BaseVehiclePartPacket vtable, and stores the supplied subtype. |
| `009d23e0` | `BaseVehiclePartPacket::Dtor` | ✅ | dtor | Restores the BaseVehiclePartPacket vtable and then the BasePacket vtable. |
| `009d2430` | `BaseVehiclePartPacket::ScalarDeletingDtor` | ✅ | dtor | Restores derived and base packet vtables, then frees the object when the deleting flag is set. |
| `009d3e10` | `BaseVehiclePartPacket::Dtor` | ✅ | dtor | Releases the owned IString buffer when present, then restores BaseVehiclePartPacket and BasePacket vtables. |
| `00b86ae0` | `BaseVehiclePartPacket::~BaseVehiclePartPacket` | 🟢 | dtor | Base destructor body used during unwinding around the vehicle-part packet constructors: resets this object to BaseVehiclePartPacket::vftable and then to BasePacket::vftable (no other work). |
| `00b86b90` | `BaseVehiclePartPacket_DestroyBase` | 🟢 | dtor | Second base-destructor body of the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then to BasePacket::vftable. |
| `00b86c40` | `BaseVehiclePartPacket_DestroyBase2` | 🟢 | dtor | Base-destructor body for the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then BasePacket::vftable. |
| `00b86cf0` | `BaseVehiclePartPacket_DestroyBase3` | 🟢 | dtor | Base-destructor body for the vehicle-part packet hierarchy: resets the object to BaseVehiclePartPacket::vftable and then BasePacket::vftable. |
