# Class: BaseClientUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089d120` | `BaseClientUpdatePacketCtor` | ✅ | ctor | Constructor of BaseClientUpdatePacket for sub-opcode 0: installs BasePacket::vftable, stores the family opcode 0x26 (38) at +4, installs BaseClientUpdatePacket::vftable and clears the sub-opcode field at +8. |
| `0089d180` | `BaseClientUpdatePacketCtorWithSubOpcode` | ✅ | ctor | Constructor of BaseClientUpdatePacket with an explicit sub-opcode: installs BasePacket::vftable, stores the family opcode 0x26 (38) at +4, installs BaseClientUpdatePacket::vftable and stores param_2 in the sub-opcode fi… |
| `0089d1f0` | `ClientUpdateBaseDtor_89d1f0` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; cited by 85 EH unwind funclets. |
| `0089d240` | `BaseClientUpdatePacketScalarDtor` | ✅ | dtor | Scalar deleting destructor: installs the derived then the base vtable and calls _free when bit 0 of param_2 is set. |
| `0089d310` | `ClientUpdateBaseDtor_89d310` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing a hitpoints update. |
| `0089d3c0` | `ClientUpdateBaseDtor_89d3c0` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing a mana update. |
| `0089d590` | `ClientUpdateBaseDtor_89d590` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used for the update packet between ItemAdd and Storage. |
| `0089d670` | `ClientUpdateBaseDtor_89d670` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing the storage update. |
| `0089d750` | `ClientUpdateBaseDtor_89d750` | ✅ | dtor | Destructor body: installs BaseClientUpdatePacket::vftable then BasePacket::vftable; the emission used after parsing the item-delete update. |
