# Class: PlayerUpdatePacketCustomizationChange

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091d5f0` | `PlayerUpdatePacketCustomizationChange::Ctor` | ✅ | ctor | Constructor: runs FUN_0089fbb0(0x27) (BasePlayerUpdatePacket with sub-opcode 39), installs PlayerUpdatePacketCustomizationChange::vftable, sets up the SoeUtil::List<PlayerCustomizationData> at +0x14 and stores 1 into th… |
| `0091d660` | `PlayerUpdatePacketCustomizationChange::~PlayerUpdatePacketCustomizationChange` | ✅ | dtor | Destructor: drains the SoeUtil::List<PlayerCustomizationData> at +0x20 with FUN_008fbe20 and restores BasePlayerUpdatePacket::vftable then BasePacket::vftable. |
| `0091d6f0` | `PlayerUpdatePacketCustomizationChange::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the packet destructor 0091d660 and frees this when bit 0 of the flag argument is set. |
