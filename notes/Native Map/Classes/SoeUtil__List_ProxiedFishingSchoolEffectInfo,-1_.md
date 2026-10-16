# Class: SoeUtil::List<ProxiedFishingSchoolEffectInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd0cd0` | `FishingSchoolEffectListAllocNode` | ✅ | method | Tries the current and global allocators, then allocates 0x10 bytes aligned to four for a school-effect list node. |
| `00cd0d10` | `FishingSchoolEffectListFreeNode` | ✅ | method | Passes the effect-info node pointer to the aligned free routine with alignment four. |
| `00cd11a0` | `FishingSchoolEffectListDtor` | ✅ | dtor | Restores the typed effect-info list vtable and clears all linked nodes. |
| `00cd11e0` | `FishingSchoolEffectListScalarDeletingDtor` | ✅ | dtor | Restores the typed effect-info list vtable, clears nodes, and frees the allocation when the deleting flag is set. |
