# Class: RewardBundleBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091c110` | `RewardBundleBase::Ctor` | ✅ | ctor | Constructor of the reward-bundle data model: calls FUN_008cf6e0, installs RewardBundleBase::vftable, zeroes/initializes the fields from +0x38 to +0xb4 (including the reward list head/tail/count at +0x38/+0x3c/+0x40, the… |
| `0091c260` | `RewardBundleBase::Set_0x90` | ✅ | virtual | Virtual setter: stores the argument into this+0x90 and returns. |
| `0091c270` | `RewardBundleBase::Get_0x90` | ✅ | virtual | Virtual getter: returns the dword at this+0x90. |
| `0091c280` | `RewardBundleBase::Add_0x48` | ✅ | virtual | Virtual accumulator: adds the argument to the dword at this+0x48. |
| `0091c290` | `RewardBundleBase::GetXpRewardAmount` | ✅ | virtual | Virtual getter: returns the dword at this+0x48, the reward XP/stars award amount. |
| `0091c2a0` | `RewardBundleBase::Set_0x48` | ✅ | virtual | Virtual setter: stores the argument into this+0x48. |
| `0091c2b0` | `RewardBundleBase::Add_0x4c` | ✅ | virtual | Virtual accumulator: adds the argument to the dword at this+0x4c. |
| `0091c2c0` | `RewardBundleBase::Get_0x4c` | ✅ | virtual | Virtual getter: returns the dword at this+0x4c. |
| `0091c2d0` | `RewardBundleBase::Add_0x50` | ✅ | virtual | Virtual accumulator: adds the argument to the dword at this+0x50. |
| `0091c2e0` | `RewardBundleBase::GetCoinRewardAmount` | ✅ | virtual | Virtual getter: returns the dword at this+0x50, the reward coin award amount. |
| `0091c2f0` | `RewardBundleBase::Set_0x50` | ✅ | virtual | Virtual setter: stores the argument into this+0x50. |
| `0091c300` | `RewardBundleBase::Set_0x64` | ✅ | virtual | Virtual setter: stores the argument into this+0x64 (printed as +100). |
| `0091c310` | `RewardBundleBase::Set_0x68` | ✅ | virtual | Virtual setter: stores the argument into this+0x68. |
| `0091c320` | `RewardBundleBase::Get_0x70` | ✅ | virtual | Virtual getter: returns the dword at this+0x70. |
| `0091c330` | `RewardBundleBase::Set_0x70` | ✅ | virtual | Virtual setter: stores the argument into this+0x70. |
| `0091c340` | `RewardBundleBase::Get_0x6c` | ✅ | virtual | Virtual getter: returns the dword at this+0x6c. |
| `0091c350` | `RewardBundleBase::Set_0x6c` | ✅ | virtual | Virtual setter: stores the argument into this+0x6c. |
| `0091c360` | `RewardBundleBase::GetByte_0x44` | ✅ | virtual | Virtual getter: returns the byte at this+0x44. |
| `0091c370` | `RewardBundleBase::SetByte_0x44` | ✅ | virtual | Virtual setter: stores the byte argument at this+0x44. |
| `0091c380` | `RewardBundleBase::GetRewardKind` | ✅ | virtual | Virtual getter: returns the dword at this+0x54, the reward kind used to choose item-notification behaviour. |
| `0091c390` | `RewardBundleBase::Set_0x54` | ✅ | virtual | Virtual setter: stores the argument into this+0x54. |
| `0091c3a0` | `RewardBundleBase::Get_0x5c` | ✅ | virtual | Virtual getter: returns the dword at this+0x5c. |
| `0091c3b0` | `RewardBundleBase::Set_0x5c` | ✅ | virtual | Virtual setter: stores the argument into this+0x5c. |
| `0091c3c0` | `RewardBundleBase::Get_0x60` | ✅ | virtual | Virtual getter: returns the dword at this+0x60. |
| `0091c3d0` | `RewardBundleBase::Set_0x60` | ✅ | virtual | Virtual setter: stores the argument into this+0x60. |
| `0091c3e0` | `RewardBundleBase::Get_0x58` | ✅ | virtual | Virtual getter: returns the dword at this+0x58. |
| `0091c3f0` | `RewardBundleBase::Set_0x58` | ✅ | virtual | Virtual setter: stores the argument into this+0x58. |
| `0091c400` | `RewardBundleBase::GetPtr_0x94` | ✅ | virtual | Virtual accessor: returns the address this+0x94, the head of one of the bundle's StringFixed lists. |
| `0091c410` | `RewardBundleBase::SetByte_0x74` | ✅ | virtual | Virtual setter: stores the byte argument at this+0x74. |
| `0091c440` | `RewardBundleBase::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls FUN_0103d010 (the reward-bundle destructor body) and then frees this when bit 0 of the flag argument is set; returns this. |
| `0091c530` | `RewardBundleBase::CopyIds_0x80_Out` | ✅ | virtual | Byte-verified (x86 decode; Ghidra prints an empty body): copies this+0x80 and this+0x84 into *param_2 and *(param_2+4), i.e. hands out the first 8-byte id pair by value. |
| `0091c550` | `RewardBundleBase::CopyIds_0x88_Out` | ✅ | virtual | Byte-verified (x86 decode; Ghidra prints an empty body): copies this+0x88 and this+0x8c into *param_2 and *(param_2+4), handing out the second 8-byte id pair by value. |
| `0091c570` | `RewardBundleBase::SetIds_0x80` | ✅ | virtual | Virtual setter: stores param_2 at this+0x80 and param_3 at this+0x84, writing the whole first 8-byte id pair. |
| `0091c590` | `RewardBundleBase::SetIds_0x88` | ✅ | virtual | Virtual setter: stores param_2 at this+0x88 and param_3 at this+0x8c, writing the whole second 8-byte id pair. |
| `0091c5b0` | `RewardBundleBase::Get_0x40` | ✅ | virtual | Virtual getter: returns the dword at this+0x40, the count of entries in the bundle's intrusive reward list. |
| `0091c5c0` | `RewardBundleBase::Get_0x38` | ✅ | virtual | Virtual getter: returns the dword at this+0x38, the head pointer of the bundle's intrusive reward list. |
| `0091c5e0` | `RewardBundleBase::AddRewardEntry` | ✅ | virtual | Appends the reward entry param_2 to the bundle's intrusive doubly-linked list (head at this+0x38, tail at this+0x3c, prev at node+4, next at node+8, count at this+0x40); when the byte at this+0x45 is set it first logs t… |
| `0091c680` | `RewardBundleBase::RemoveRewardEntry` | ✅ | virtual | Unlinks the reward entry param_2 from the bundle's intrusive list: when the node has no predecessor (node+4) the head at this+0x38 becomes node+8, otherwise the predecessor's next is updated; when the node has no succes… |
| `0091c7c0` | `RewardBundleBase::AppendStringEntry` | ✅ | virtual | Byte-verified wrapper: add ecx,0x94; call 0091c720 (append a new entry to the StringFixed<256> list at this+0x94); then mov ecx,eax and tail-jump to SoeUtil::IStringAssign (FUN_00402ce0) so the new entry receives the ca… |
| `009b9a70` | `RewardBundleBase::RewardBundleBase` | ✅ | ctor | Reward-bundle base constructor: installs RewardBundleBase::vftable, zeroes slots 0xe/0xf/0x10 and 0x24, installs the SoeUtil::List vftables for StringFixed<256> and StringFixed<512> list members with empty head/tail/cou… |
| `00c33830` | `RewardBundleBaseCtor` | ✅ | ctor | Constructs logging state, installs RewardBundleBase vtable, zeroes fields and both embedded lists, then initializes a following member through FUN_0103a1e0. |
