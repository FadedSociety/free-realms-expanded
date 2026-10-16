# Class: SoeUtil::HashListMap<int,EncounterParticipant,25,5>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a24c80` | `EncounterParticipantHashMapInit` | ✅ | function | Initializes map fields, clears its 25-bucket table and links fixed-size nodes into a free chain. |
| `00a25700` | `HashListMap::PooledNodeAllocateAdjustor` | ✅ | virtual | Adjusts ECX by 0x78, then jumps to 00a25620; the target replenishes and pops the pool free chain. |
| `00a25710` | `HashListMap::AddAdjustor` | ✅ | virtual | Adds 0x78 to ECX, then jumps to shared body 00a25690. |
| `00a25c20` | `EncounterParticipantBoundedHashMapDtor` | ✅ | dtor | Destroys entries and block storage, resets block fields, then restores base map vtable and clears entries. |
| `00a25cf0` | `EncounterParticipantBoundedHashMapScalarDeletingDtor` | ✅ | virtual | Calls map cleanup and frees object when deletion flag bit zero is set. |
