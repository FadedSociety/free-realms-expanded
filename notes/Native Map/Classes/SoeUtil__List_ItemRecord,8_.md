# Class: SoeUtil::List<ItemRecord,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a03670` | `AdjustThisPlus0x10AndJump0080e920 → List_ItemRecord_8_AllocNodeThunk` | ✅ | method | 8-byte adjustor thunk: adds 0x10 to ECX and jumps to 0080e920. |
| `00a03680` | `AdjustThisPlus0x10AndJump0080e990 → List_ItemRecord_8_FreeNodeThunk` | ✅ | method | 8-byte adjustor thunk: adds 0x10 to ECX and jumps to 0080e990. |
| `00a03780` | `List_ItemRecord_8_ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor of SoeUtil::List<ItemRecord,8>: runs the hashed-list destructor body 00a03690 and frees this when the delete flag is set. |
