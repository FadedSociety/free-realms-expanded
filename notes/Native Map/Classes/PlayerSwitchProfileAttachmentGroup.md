# Class: PlayerSwitchProfileAttachmentGroup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00964ba0` | `PlayerSwitchProfileAttachmentGroup::Constructor` | ✅ | ctor | Adjusts nonzero param_4 by +0x54, calls FUN_00bf6d60 with the supplied constructor values, installs the PlayerSwitchProfileAttachmentGroup vtable, and stores three scalar fields. |
| `00964c50` | `PlayerSwitchProfileAttachmentGroup::GetAdjustedField174` | ✅ | virtual | Returns zero when this+0x174 is zero; otherwise returns this+0x174 minus 0x54. |
| `00964c70` | `PlayerSwitchProfileAttachmentGroup::UpdateBaseAndNotify` | ✅ | virtual | Calls FUN_0072fb60 with its argument, calls FUN_01051150, and when that returns nonzero calls FUN_00bfb620. |
| `00964ca0` | `PlayerSwitchProfileAttachmentGroup::NotifyWhenReady` | ✅ | virtual | Calls FUN_01051150 and, only when it returns nonzero, calls FUN_00bfb620. |
| `00964cc0` | `PlayerSwitchProfileAttachmentGroup::FilterField174` | ✅ | virtual | Reads this+0x174; returns the value only when it is nonzero and not 0x54, otherwise returns zero. |
| `00964ce0` | `PlayerSwitchProfileAttachmentGroup::CanApplyProfileChange ⚠downgrade` | 🟢 | virtual | Returns zero if FUN_01051150 is false; returns one when its stored node field is zero; otherwise checks a context field and hash-table lookups, then returns the selected node's virtual slot +0x1c result or zero. |
| `00964d50` | `PlayerSwitchProfileAttachmentGroup::Destructor` | ✅ | dtor | Restores the PlayerSwitchProfileAttachmentGroup vtable and calls FUN_00964b00 to destroy its attachment-group base. |
| `00964da0` | `PlayerSwitchProfileAttachmentGroup::ScalarDeletingDestructor` | ✅ | dtor | Restores the class vtable, calls FUN_00964b00 for base cleanup, and frees param_1 only when flag bit 0 is set. |
