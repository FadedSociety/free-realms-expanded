# Class: PlayerUpdatePacketReplaceBaseModel

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1850` | `PlayerUpdatePacketReplaceBaseModel::~PlayerUpdatePacketReplaceBaseModel` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x31 and also runs from EH funclet 01591772. |
| `008c5020` | `PlayerUpdatePacketReplaceBaseModel::Constructor` | ✅ | ctor | PlayClient case 0x31 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x31; this body installs its vtable and zeros dword +0x1c. |
| `008c5080` | `PlayerUpdatePacketReplaceBaseModel::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
