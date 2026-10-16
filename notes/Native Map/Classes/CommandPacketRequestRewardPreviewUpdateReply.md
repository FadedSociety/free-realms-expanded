# Class: CommandPacketRequestRewardPreviewUpdateReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a98930` | `CommandPacketRequestRewardPreviewUpdateReply::~CommandPacketRequestRewardPrevie…` | ✅ | dtor | Stores CommandPacketRequestRewardPreviewUpdateReply::vftable, calls 0103d010 on the embedded reward-bundle member (0103d010's body restores RewardBundleBase::vftable and releases a List<StringFixed<512>> member; the dec… |
| `00a989a0` | `CommandPacketRequestRewardPreviewUpdateReply::ScalarDeletingDtor` | 🟢 | dtor | Calls 00a98930 on this, then if the low bit of the flag byte is set calls _free on the object; returns the object pointer. |
| `00aa1d00` | `CommandPacketRequestRewardPreviewUpdateReply::CommandPacketRequestRewardPreview…` | ✅ | ctor | Constructor: calls the command-family constructor FUN_008a2820 with sub-opcode 0x26 (38) and 0, stores CommandPacketRequestRewardPreviewUpdateReply::vftable at +0 and then constructs one member object through FUN_0091c1… |
