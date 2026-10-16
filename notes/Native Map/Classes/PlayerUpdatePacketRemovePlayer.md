# Class: PlayerUpdatePacketRemovePlayer

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089fce0` | `PlayerUpdatePacketRemovePlayer::~PlayerUpdatePacketRemovePlayer` | ✅ | dtor | Destructor body of the sub-opcode-3 packet: restores PlayerUpdatePacketRemovePlayer::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. No fields touched, no deallocation. |
| `0089fd40` | `PlayerUpdatePacketRemovePlayer::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores PlayerUpdatePacketRemovePlayer, BasePlayerUpdatePacket and BasePacket vtables in an SEH frame, and calls _free(this) when param_2 bit 0 is set. |
| `008b3840` | `— ⚠reject` | ⚪ | ctor | Calls the opcode-35 family constructor 0089fbb0(3) (which stores BasePacket id 0x23 with sub-opcode 3), installs PlayerUpdatePacketRemovePlayer::vftable and zeroes the dword at +0xc. |
| `008b38a0` | `— ⚠reject` | ⚪ | ctor | Calls the opcode-35 family constructor 0089fbb0(3), installs PlayerUpdatePacketRemovePlayer::vftable and stores param_2 into the dword at +0xc; the only call site (0x8c4afa inside 008c4ad0) passes the constant 1. |
