# Class: ReferenceDataPacketClientProfileData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a39bb0` | `ReferenceDataPacketClientProfileData::ReferenceDataPacketClientProfileData` | ✅ | ctor | Constructor of the reference-data sub-opcode 3 packet: calls the family constructor FUN_008a4550 with sub-opcode 3, installs ReferenceDataPacketClientProfileData::vftable at +0 and stores the argument (the client profil… |
| `00a39c20` | `ReferenceDataPacketClientProfileData::~ReferenceDataPacketClientProfileData` | 🟢 | dtor | Destructor body: writes ReferenceDataPacket::vftable at +0 and then BasePacket::vftable at +0, i.e. it unwinds the packet object back to its base classes; no other members are released. |
| `00a39d90` | `ReferenceDataPacketClientProfileData::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: under an SEH frame installs ReferenceDataPacket::vftable then BasePacket::vftable at +0, and calls _free(this) when bit 0 of the flag argument is set; returns this otherwise. |
| `00a3a900` | `ReferenceDataPacketClientProfileData_Parse` | ✅ | handler | Receive path for reference-data sub-opcode 3: constructs a ReferenceDataPacketClientProfileData in its own SEH frame over the supplied table pointer, builds a bounded reader over (param_2, param_2+param_3), reads two 16… |
