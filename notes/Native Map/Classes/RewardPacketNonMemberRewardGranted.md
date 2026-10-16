# Class: RewardPacketNonMemberRewardGranted

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b893a0` | `RewardPacketNonMemberRewardGranted::RewardPacketNonMemberRewardGranted` | ✅ | ctor | Constructor of the subtype-7 granted-bank packet: runs the base constructor with subtype 7 (00b88b40), installs RewardPacketNonMemberRewardGranted::vftable, zeroes +0x1c and initializes the embedded SoeUtil::List<Deferr… |
| `00b89410` | `RewardPacketNonMemberRewardGranted_Destroy` | 🟢 | dtor | Destructor body of the granted-bank packet: reinstalls the list vtable at +0x20, clears the embedded deferred-stars list through 00b88f80 and then destroys the reward packet base via 00b88bc0. |
| `00b894a0` | `RewardPacketNonMemberRewardGranted_ReadBody` | ✅ | method | Body reader for the subtype-7 granted packet: calls the base header reader 00b89190, reads one 32-bit value into +0x1c (granted coins) with bounds handling, then reads the row list through 00b89340 into the embedded lis… |
| `00b89500` | `RewardPacketNonMemberRewardGranted::ScalarDeletingDtor` | ✅ | dtor | Vtable slot 0 of the granted-bank packet: runs the destructor body 00b89410 and frees the packet through _free when bit 0 of param_2 is set. |
| `00b89c00` | `RewardPacketNonMemberRewardGranted_Read` | 🟢 | method | Entry point that reads a subtype-7 granted packet body from a raw buffer: constructs the bounded reader over (param_2, param_2+param_3), calls the body reader 00b894a0 and returns 1 when no error was flagged and the buf… |
