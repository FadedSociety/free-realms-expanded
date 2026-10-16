# Class: VehicleUpdateItemPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4dd0` | `VehicleUpdateItemPacket::VehicleUpdateItemPacket` | ✅ | ctor | Constructor: calls FUN_008a4c40(2), installs VehicleUpdateItemPacket::vftable, zeroes payload dwords at +0xc and +0x10, then calls FUN_0101cd40 with ECX pointing to the subobject at this+0x14. The /GS frame-cookie value… |
| `008a4e40` | `VehicleUpdateItemPacket::~VehicleUpdateItemPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable and returns; the only caller is EH unwind funclet 0159334c. |
| `008b7f80` | `VehicleUpdateItemPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
