# Class: BaseLoyaltyRewardPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a5190` | `BaseLoyaltyRewardPacket::BaseLoyaltyRewardPacket` | ✅ | ctor | Loyalty-reward family constructor: installs BasePacket::vftable, stores opcode 173 (0xad) at +4, installs BaseLoyaltyRewardPacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a5200` | `BaseLoyaltyRewardPacket::~BaseLoyaltyRewardPacket` | ✅ | dtor | Destructor body: restores BaseLoyaltyRewardPacket::vftable then BasePacket::vftable; four EH funclets (0157e290 and neighbours) call it. |
| `008a5250` | `BaseLoyaltyRewardPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores BaseLoyaltyRewardPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
