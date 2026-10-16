# Class: BaseVehicleRacePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b99270` | `BaseVehicleRacePacket::BaseVehicleRacePacket_Default` | ✅ | ctor | Family constructor without a subtype word: stores BasePacket::vftable, writes the opcode 0x21 (33) into the packet id slot at +4, stores BaseVehicleRacePacket::vftable and zeroes the subtype field at +8. |
| `00b992d0` | `BaseVehicleRacePacket::BaseVehicleRacePacket_WithSubtype` | ✅ | ctor | Family constructor: stores BasePacket::vftable, writes the opcode 0x21 (33) into the packet id slot at +4, stores BaseVehicleRacePacket::vftable and stores the subtype argument param_2 at +8; no payload. |
| `00b99340` | `BaseVehicleRacePacket::~BaseVehicleRacePacket` | ✅ | dtor | Destructor body of the vehicle race packet base: stores BaseVehicleRacePacket::vftable at this+0 and then BasePacket::vftable; touches no fields and frees nothing. |
| `00b99390` | `BaseVehicleRacePacket::ScalarDeletingDestructor` | ✅ | dtor | MSVC scalar deleting destructor: stores BaseVehicleRacePacket::vftable then BasePacket::vftable inside an SEH frame and calls _free(this) when bit 0 of param_2 is set. |
| `00b996b0` | `BaseVehicleRacePacket_DtorEmission_A` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99700` | `BaseVehicleRacePacket_DtorEmission_B` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99940` | `BaseVehicleRacePacket_DtorEmission_C` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99a40` | `BaseVehicleRacePacket_DtorEmission_D` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b99bf0` | `BaseVehicleRacePacket_DtorEmission_E` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9a300` | `BaseVehicleRacePacket_DtorEmission_F` | 🟢 | dtor | Destructor body that stores BaseVehicleRacePacket::vftable then BasePacket::vftable; no fields touched. |
| `00b9d230` | `BaseVehicleRacePacket::CopyHeaderFrom` | ✅ | method | Copy-initialises a packet header in place: stores BasePacket::vftable into *this, copies param_2[1] into this[1]; stores BaseVehicleRacePacket::vftable into *this and copies param_2[2] into this[2] (the two header words… |
