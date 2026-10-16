# Class: CommandPacketConfirmFriendResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a0a500` | `CommandPacketConfirmFriendResponse::CommandPacketConfirmFriendResponse ⚠downgra…` | 🟢 | ctor | Constructor: runs the shared command base constructor with sub-opcode 0x11 (17), installs the CommandPacketConfirmFriendResponse vtable, stores three arguments at +0x10/+0x14/+0x18 and initialises a trailing member from… |
| `00a0a580` | `CommandPacketConfirmFriendResponse::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the base command teardown 00a0a4a0 and frees this when the delete flag is set. |
