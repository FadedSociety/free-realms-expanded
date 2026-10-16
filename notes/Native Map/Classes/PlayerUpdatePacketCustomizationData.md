# Class: PlayerUpdatePacketCustomizationData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091d4d0` | `— ⚠reject` | ⚪ | ctor | Constructor: runs FUN_0089fbb0(0x41) (BasePlayerUpdatePacket with sub-opcode 65) and installs PlayerUpdatePacketCustomizationData::vftable plus the SoeUtil::List<PlayerCustomizationData> vtable at +0x10. |
| `0091d540` | `PlayerUpdatePacketCustomizationData::~PlayerUpdatePacketCustomizationData` | ✅ | dtor | Destructor body: drains the SoeUtil::List<PlayerCustomizationData> at +0x10 with FUN_008fbe20 and restores PlayerUpdatePacketCustomizationData::vftable, BasePlayerUpdatePacket::vftable and BasePacket::vftable in sequenc… |
| `0091d5d0` | `PlayerUpdatePacketCustomizationData::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the packet destructor 0091d540 and frees this when bit 0 of the flag argument is set. |
