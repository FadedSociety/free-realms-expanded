# Class: PlayerUpdatePacketPlayCompositeEffect

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0af0` | `PlayerUpdatePacketPlayCompositeEffect::~PlayerUpdatePacketPlayCompositeEffect` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x10 and EH funclet 015915c2. |
| `008c4b30` | `PlayerUpdatePacketPlayCompositeEffect::Constructor` | ✅ | ctor | PlayClient case 0x10 constructs this packet. Base constructor sets family opcode 0x23/subcode 0x10; this body installs its vtable, copies defaults into +0x10..+0x1c and +0x30..+0x3c, zeros +0x20..+0x28, and sets byte +0… |
| `008c4bd0` | `PlayerUpdatePacketPlayCompositeEffect::ScalarDeletingDestructor` | ✅ | dtor | Restores BasePlayerUpdatePacket and BasePacket vtables; frees this when deleting-flag bit 0 is set, otherwise returns this. |
