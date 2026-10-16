# Class: PlayerUpdatePacketRenamePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d09a0` | `PlayerUpdatePacketRenamePlayer::~PlayerUpdatePacketRenamePlayer` | 🟢 | dtor | Destructor body of a BasePlayerUpdatePacket-derived packet: destroys the member object at this+0x18 through FUN_0070a790, then rewrites the object vtable to BasePlayerUpdatePacket::vftable and finally BasePacket::vftabl… |
| `008dcc10` | `PlayerUpdatePacketRenamePlayer::PlayerUpdatePacketRenamePlayer` | ✅ | ctor | Packet constructor: calls family initializer 0089fbb0 with sub-opcode 0x13 (19, family 35), installs PlayerUpdatePacketRenamePlayer::vftable and calls the embedded member initializer 0070bac0 with ECX=this+0x18. |
| `008dcc70` | `PlayerUpdatePacketRenamePlayer::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d09a0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
