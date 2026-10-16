# Class: ProxiedCharacterAttachmentGroupManager

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00957f70` | `ProxiedCharacterAttachmentGroupManager::ScalarDeletingDtor ?` | 🟡 | dtor | Calls 00bf14c0 and frees this allocation when deletion flag bit 0 is set. |
| `00bf0f70` | `ProxiedCharacterAttachmentGroupManager::IsEmpty` | ✅ | virtual | Returns true when the 32-bit field at object offset 0x118 is zero. |
| `00bf11e0` | `ProxiedCharacterAttachmentGroupManager::RemoveGroup ⚠downgrade` | 🟢 | virtual | Rejects an ineligible group, optionally removes matching global attachments, optionally calls a manager reset slot, unlinks the matching node from the manager list, and updates a state byte. |
| `00bf1290` | `ProxiedCharacterAttachmentGroupManager::RemoveGroupsForId ⚠downgrade` | 🟢 | virtual | Finds the hash bucket for a group ID, skips entries with other group IDs, then removes matching entries while their secondary ID at offset 0xb4 equals the requested ID. |
| `00bf1320` | `ProxiedCharacterAttachmentGroupManager::UpdateVisibility` | ✅ | virtual | Updates a global visibility state from manager offset 0x104, then walks linked groups and disables the associated value for groups whose flag at offset 0x168 is set. |
| `00bf1420` | `ProxiedCharacterAttachmentGroupManager::Ctor` | ✅ | ctor | Constructs a base object, installs the manager vtable, clears manager state, stores a parameter, and initializes the embedded attachment-group list and count. |
| `00bf14c0` | `ProxiedCharacterAttachmentGroupManager::Dtor` | ✅ | dtor | Restores the manager vtable, restores and clears the embedded list, then calls the base cleanup routine. |
| `00bf1530` | `ProxiedCharacterAttachmentGroupManager::ProcessGroup` | ✅ | virtual | Looks up attachment groups by GUID, invokes callbacks for active matching entries, resolves dual-wield slot definitions and logs missing definitions, then marks the selected attachment group active and processes it. |
