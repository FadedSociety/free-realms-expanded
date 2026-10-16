# Class: CommandPacketConfirmFriendRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a9c270` | `CommandPacketConfirmFriendRequest::~CommandPacketConfirmFriendRequest` | ✅ | dtor | Stores CommandPacketConfirmFriendRequest::vftable, then cleans up the member object at this+0x18 through 0070a790 (the bytes load lea ecx,[esi+0x18] before the call; 0070a790 is a cleanup routine shared by hundreds of E… |
| `00a9c2e0` | `CommandPacketConfirmFriendRequest::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a9c270 (the ConfirmFriendRequest destructor body) on this, then if the low bit of the flag byte is set calls _free on the object and returns it. |
| `00a9de90` | `CommandPacketConfirmFriendRequest::CommandPacketConfirmFriendRequest` | ✅ | ctor | Constructor of the client-side confirm-friend-request command packet: calls the command-family constructor FUN_008a2820 with sub-opcode 0x10 (16) and 0, installs CommandPacketConfirmFriendRequest::vftable at +0, copies … |
