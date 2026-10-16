# Class: GroupPacketRenamePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3630` | `GroupPacketRenamePlayer::~GroupPacketRenamePlayer` | 🟢 | dtor | Destructor body: restores BaseGroupPacket::vftable then BasePacket::vftable; the group dispatcher 0093daf0 destroys the rename packet with it immediately after constructing GroupPacketRenamePlayer (sub 11) and applying … |
| `008d1800` | `GroupPacketRenamePlayer::~GroupPacketRenamePlayer` | 🟢 | dtor | Destructor body of the group rename-player packet: destroys the member at +0x18 through FUN_0070a790, then rewrites the object vtable to BaseGroupPacket::vftable and BasePacket::vftable inside an SEH cookie frame. |
| `008dd1b0` | `GroupPacketRenamePlayer::GroupPacketRenamePlayer` | ✅ | ctor | Packet constructor: calls the group family initializer 008a35c0 with sub-opcode 0xb (11), installs GroupPacketRenamePlayer::vftable and calls embedded member initializer 0070bac0 with ECX=this+0x18. |
| `008dd210` | `GroupPacketRenamePlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d1800 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
