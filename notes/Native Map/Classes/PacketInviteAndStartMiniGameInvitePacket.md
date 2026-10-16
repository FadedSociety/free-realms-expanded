# Class: PacketInviteAndStartMiniGameInvitePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a878c0` | `— ⚠reject` | ⚪ | ctor | Constructor of the minigame-invite packet subclass: runs the PacketInviteAndStartMiniGamePacket family constructor 008b0fb0(1) (which installs BasePacket's vtable and stores family id 180 plus sub-opcode 1), stores the … |
| `00a87920` | `PacketInviteAndStartMiniGameInvitePacketScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]) of PacketInviteAndStartMiniGameInvitePacket: rewrites the object's vtable to ClientGateway::BasePacket and frees it with _free when the delete flag bit 0 is set. |
| `00a97cc0` | `PacketInviteAndStartMiniGameInvitePacket::SendFields` | 🟢 | method | Thiscall with five dword arguments and a byte flag. After callee 00a878c0 has constructed the packet at buffer+0, fills buffer+0x10 through buffer+0x20 with the five dwords in argument order and buffer+0x28 with the byt… |
