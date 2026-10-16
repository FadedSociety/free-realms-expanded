# Class: CommandPacketRequestRewardPreviewUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2de0` | `CommandPacketRequestRewardPreviewUpdate::~CommandPacketRequestRewardPreviewUpda…` | 🟢 | dtor | Destructor body: restores CommandPacketRequestRewardPreviewUpdate::vftable, BaseCommandPacket::vftable and BasePacket::vftable; its only non-self caller is EH funclet 01591d28. |
| `008a2e40` | `CommandPacketRequestRewardPreviewUpdate::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the reward-preview, command and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `008b6cb0` | `CommandPacketRequestRewardPreviewUpdate::CommandPacketRequestRewardPreviewUpdat…` | 🟢 | ctor | Calls the command family constructor 008a2820(0x25, 0), installs CommandPacketRequestRewardPreviewUpdate::vftable, stores param_2 into +0x10 and the global pair DAT_01be4850/DAT_01be4854 into +0x18/+0x1c. |
