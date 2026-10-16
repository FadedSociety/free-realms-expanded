# Class: MiniGameRewardStatusPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b9d10` | `MiniGameRewardStatusPacket::MiniGameRewardStatusPacket` | ✅ | ctor | Constructs the family-39 sub-opcode 22 (0x16) reward-status packet: calls FUN_008a4420(0x16,param_2,-1,-1), installs MiniGameRewardStatusPacket::vftable, initialises two reward-bundle entries at +0x18 and +0xd0 (FUN_009… |
| `009b9e00` | `MiniGameRewardStatusPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor for the 39/22 reward-status packet: calls the packet destructor body 009b31c0, the same destructor the handler 009ba720 uses to close its in-place temporary. Then frees the object through _fre… |
