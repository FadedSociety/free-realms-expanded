# Class: BattleMagesPacketUpdateData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b6fb80` | `BattleMagesPacketUpdateData::BattleMagesPacketUpdateData` | ✅ | ctor | Constructor for sub-opcode 2: calls the shared base constructor with 2 and stores BattleMagesPacketUpdateData::vftable at +0; it initialises no other field. |
| `00b6fbe0` | `BattleMagesPacketUpdateData::ScalarDeletingDestructor` | ✅ | dtor | Virtual deleting destructor at the UpdateData packet's vtable slot 0: writes BaseBattleMagesPacket::vftable, then BasePacket::vftable, and calls _free(this) when bit 0 of the flag argument is set. |
