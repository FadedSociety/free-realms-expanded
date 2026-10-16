# Class: CommandPacketActivateProfileFailed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2ca0` | `CommandPacketActivateProfileFailed::CommandPacketActivateProfileFailed` | ✅ | ctor | Calls FUN_008a2820(0x1a,0), installs CommandPacketActivateProfileFailed::vftable, and zeroes payload dwords param_1[4] and param_1[5]; caller 0092f260 fills both before sending. |
| `008a2d10` | `CommandPacketActivateProfileFailed::~CommandPacketActivateProfileFailed` | 🟢 | dtor | Destructor body: restores the derived vtable, then BaseCommandPacket::vftable and BasePacket::vftable. |
| `008a2d70` | `CommandPacketActivateProfileFailed::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the ActivateProfileFailed, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
