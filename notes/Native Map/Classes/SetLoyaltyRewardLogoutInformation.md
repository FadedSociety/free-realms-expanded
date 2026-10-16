# Class: SetLoyaltyRewardLogoutInformation

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a52c0` | `SetLoyaltyRewardLogoutInformation::SetLoyaltyRewardLogoutInformation` | ✅ | ctor | Constructor: calls FUN_008a5190(0) so the packet is opcode 173 with sub-opcode 0, installs SetLoyaltyRewardLogoutInformation::vftable and zeroes +0xc and +0x10; its only caller is 0090e7b0. |
| `008a5330` | `SetLoyaltyRewardLogoutInformation::~SetLoyaltyRewardLogoutInformation` | ✅ | dtor | Destructor body: restores SetLoyaltyRewardLogoutInformation::vftable, then BaseLoyaltyRewardPacket::vftable, then BasePacket::vftable; its only caller is EH funclet 0158c2d0. |
| `008a5390` | `SetLoyaltyRewardLogoutInformation::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores the logout-report, loyalty-reward and BasePacket vtables, then calls _free(this) when param_2 bit 0 is set. |
