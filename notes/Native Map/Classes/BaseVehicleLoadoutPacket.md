# Class: BaseVehicleLoadoutPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4c40` | `BaseVehicleLoadoutPacket::BaseVehicleLoadoutPacket` | ✅ | ctor | Vehicle packet-family constructor: installs BasePacket::vftable, stores opcode 137 (0x89) at +4, installs BaseVehicleLoadoutPacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a4cb0` | `BaseVehicleLoadoutPacket::~BaseVehicleLoadoutPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable and returns; nine EH funclets (0157dfe0 and neighbours) call it. |
| `008b7f00` | `BaseVehicleLoadoutPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
