# Class: PlayerUpdatePacketRemovePlayerGracefully

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089fdb0` | `PlayerUpdatePacketRemovePlayerGracefully::~PlayerUpdatePacketRemovePlayerGracef…` | ✅ | dtor | Destructor body of the sub-opcode-1 packet: restores PlayerUpdatePacketRemovePlayerGracefully::vftable, then PlayerUpdatePacketRemovePlayer::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `0089fe10` | `PlayerUpdatePacketRemovePlayerGracefully::ScalarDeletingDestructor` | 🟢 | dtor | Thin deleting destructor: calls FUN_0089fdb0 on this and then, when param_2 bit 0 is set, calls _free(this). |
| `008c4ad0` | `PlayerUpdatePacketRemovePlayerGracefully::Constructor` | ✅ | ctor | Calls 0x008b38a0 with this in ECX and stack argument 1, installs PlayerUpdatePacketRemovePlayerGracefully vtable, zeros dword +0x24, and returns this. Its caller is PlayClient dispatcher 0x0092f460. |
