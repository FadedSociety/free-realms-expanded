# Class: ProxiedCharacterAttachmentGroup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0095ff70` | `ProxiedCharacterAttachmentGroup::GetByte9D` | ✅ | virtual | Returns the byte stored at this+0x9d. |
| `00964b00` | `ProxiedCharacterAttachmentGroup::Destructor` | ✅ | dtor | Restores the ProxiedCharacterAttachmentGroup vtable, installs the integer-list vtable for an embedded member, calls FUN_007798c0, then destroys the NewHatHairAttachmentGroup base/member through FUN_0095fec0. |
| `00964b70` | `ProxiedCharacterAttachmentGroup::GetField174` | ✅ | virtual | Returns the dword at this+0x174. |
| `00964b80` | `ProxiedCharacterAttachmentGroup::ScalarDeletingDestructor` | ✅ | dtor | Calls FUN_00964b00 for class cleanup and frees param_1 only when flag bit 0 is set. |
| `00bf6980` | `AttachmentGroupSlot4Forward ?` | 🟡 | virtual | Shared vtable slot [4] of ProxiedCharacterAttachmentGroup, PlayerSwitchProfileAttachmentGroup and GuiModelAttachmentGroup. Forwards its stack arguments to the thunk 01051140 (jumps to 00730480). Then, when arg9 == 1 and… |
| `00bf6d60` | `ProxiedCharacterAttachmentGroup::Ctor` | ✅ | ctor | Constructor body. Calls 01051490 with the forwarded arguments, stores ProxiedCharacterAttachmentGroup::vftable, stores arg4 at this+0x174, clears the byte at +0x178, stores SoeUtil::List<int,-1>::vftable at +0x17c and z… |
| `00bf6e10` | `— ⚠reject` | ⚪ | virtual | Shared vtable slot [20] of ProxiedCharacterAttachmentGroup, PlayerSwitchProfileAttachmentGroup and GuiModelAttachmentGroup. Sets this+0x6a = 1, then walks the chain at this+0x18 (next pointer at +0x190) and the attachme… |
| `00bf7240` | `— ⚠reject` | ⚪ | virtual | Shared vtable slot [17] of the three attachment group classes. Returns 1 early when its flag byte at +0x9c is clear. When the group has no entries (count at +0x20 < 1) it logs 'ProxiedCharacterAttachmentGroup has no ent… |
| `00bf75d0` | `ProxiedCharacterAttachmentGroup::CheckDuplicateSlots` | ✅ | virtual | Shared vtable slot [21] of the three attachment group classes. Builds a bool table (Array<bool,32,1>, 00bf6b70) and walks the chain at this+0x18 (slot index at node+0x17c, next at +0x190). The first entry whose slot is … |
