# Class: PlayerUpdatePacketSetSpawnerActivationEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1520` | `PlayerUpdatePacketSetSpawnerActivationEffect::~PlayerUpdatePacketSetSpawnerActi…` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x2f and also runs from EH funclet 0159157a. |
| `008c4e60` | `PlayerUpdatePacketSetSpawnerActivationEffect::Constructor` | ✅ | ctor | PlayClient case 0x2f constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x2f; this body installs its vtable and zeros dword +0x18. |
| `008c4ec0` | `PlayerUpdatePacketSetSpawnerActivationEffect::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
