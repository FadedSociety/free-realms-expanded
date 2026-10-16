# Class: ClientUpdatePacketActivateProfile

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091bfe0` | `ClientUpdatePacketActivateProfile::Ctor` | ✅ | ctor | Constructor: runs the BaseClientUpdatePacket family constructor FUN_0089d180 with sub-opcode 0x15 (21), installs ClientUpdatePacketActivateProfile::vftable, and initializes a SoeUtil::List<CharacterAttachmentData> at +0… |
| `0091c060` | `ClientUpdatePacketActivateProfile::~ClientUpdatePacketActivateProfile` | ✅ | dtor | Destructor body: installs ClientUpdatePacketActivateProfile::vftable, drains the SoeUtil::List<CharacterAttachmentData> at +0x14 through FUN_0070e670, then rewrites the vtable slots to BaseClientUpdatePacket::vftable an… |
| `0091c0f0` | `ClientUpdatePacketActivateProfile::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the packet destructor 0091c060 and then frees this when bit 0 of the flag argument is set. |
