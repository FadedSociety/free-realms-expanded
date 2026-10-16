# Class: PlayerUpdatePacketEquippedItemsChange

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091d3b0` | `PlayerUpdatePacketEquippedItemsChange::Ctor` | ✅ | ctor | Constructor: runs FUN_0089fbb0(7) (BasePlayerUpdatePacket with sub-opcode 7) and installs PlayerUpdatePacketEquippedItemsChange::vftable plus the SoeUtil::List<CharacterAttachmentData> vtable at +0x1c. |
| `0091d420` | `PlayerUpdatePacketEquippedItemsChange::~PlayerUpdatePacketEquippedItemsChange` | ✅ | dtor | Destructor body: drains the SoeUtil::List<CharacterAttachmentData> at +0x1c through FUN_0070e670 and restores PlayerUpdatePacketEquippedItemsChange::vftable, BasePlayerUpdatePacket::vftable and BasePacket::vftable in se… |
| `0091d4b0` | `PlayerUpdatePacketEquippedItemsChange::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the packet destructor 0091d420 and frees this when bit 0 of the flag argument is set. |
