# Class: SoeUtil::List<ProxiedFishingSchoolModelInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd0c60` | `FishingSchoolModelListAllocNode` | ✅ | method | Tries the current and global allocators, then allocates 0x18 bytes aligned to four for a fishing school model-info list node. |
| `00cd0ca0` | `FishingSchoolModelListFreeNode` | ✅ | method | Passes the node pointer to the aligned free routine with alignment four. |
| `00cd1190` | `FishingSchoolModelListDtor` | ✅ | dtor | Restores the typed model-info list vtable and clears all linked nodes. |
| `00cd11b0` | `FishingSchoolModelListScalarDeletingDtor` | ✅ | dtor | Restores the typed model-info list vtable, clears nodes, and frees the allocation when the deleting flag is set. |
