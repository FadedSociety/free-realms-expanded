# Class: MiniGameUpdateRewardPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b9eb0` | `MiniGameUpdateRewardPacket::MiniGameUpdateRewardPacket` | ✅ | ctor | Constructs the family-39 sub-opcode 68 (0x44) update-reward packet: calls FUN_008a4420(0x44,-1,-1,-1), installs MiniGameUpdateRewardPacket::vftable and initialises the embedded state record at +0x18 through FUN_009b9b30. |
| `009b9f20` | `MiniGameUpdateRewardPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the 39/68 update-reward packet: calls the packet destructor body 009b0a60. Then frees the object through _free when the low bit of param_2 is set (test byte [esp+8],1) and returns the poin… |
