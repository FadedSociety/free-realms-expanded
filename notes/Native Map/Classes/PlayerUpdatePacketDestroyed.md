# Class: PlayerUpdatePacketDestroyed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a16f0` | `PlayerUpdatePacketDestroyed::~PlayerUpdatePacketDestroyed` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x38 and also runs from EH funclet 015917bf. |
| `008c4f30` | `PlayerUpdatePacketDestroyed::Constructor` | ✅ | ctor | PlayClient case 0x38 constructs this subtype. Base constructor sets family opcode 0x23/subcode 0x38; this body installs its vtable, copies defaults into +0x10/+0x14, and zeros dwords +0x18 through +0x20. |
| `008c4fb0` | `PlayerUpdatePacketDestroyed::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
