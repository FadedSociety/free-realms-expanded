# Class: RewardPacketNonMemberRewardDeferred

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b88e10` | `RewardPacketNonMemberRewardDeferred::RewardPacketNonMemberRewardDeferred` | ✅ | ctor | Constructor of the subtype-6 deferred non-member reward packet: runs the base constructor with subtype 6 (00b88b40), installs RewardPacketNonMemberRewardDeferred::vftable and zeroes the three payload dwords at +0x1c, +0… |
| `00b89090` | `RewardPacketNonMemberRewardDeferred::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the subtype-6 deferred packet: runs the base destructor 00b88bc0 under an SEH frame and frees the packet through _free when bit 0 of param_2 is set. |
| `00b892b0` | `RewardPacketNonMemberRewardDeferred_ReadBody` | ✅ | method | Body reader for the subtype-6 deferred packet: calls the base header reader 00b89190 and then reads three 32-bit words into the fields at +0x1c, +0x20 and +0x24 with the usual bounds check (zero, error byte, cursor clam… |
| `00b89600` | `RewardPacketNonMemberRewardDeferred_Read` | 🟢 | method | Entry point that reads a subtype-6 deferred packet body from a raw buffer: constructs the bounded reader over (param_2, param_2+param_3), calls the body reader 00b892b0 and returns 1 when no error was flagged and the bu… |
