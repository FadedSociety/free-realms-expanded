# Class: SoeUtil::HashListMap<int,ClientActivityCategoryDefinition,128,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdf310` | `HashListMap_ClientActivityCategoryDefinition_Ctor` | ✅ | ctor | Constructor: stores SoeUtil::HashListMap<int,ClientActivityCategoryDefinition,128,-1>::vftable at object+0, zeroes the pointer/count fields at +4..+0x10 and the 128 bucket heads at +0x14 with one _memset of 0x200 bytes. |
| `00be6a80` | `HashListMapClientActivityCategoryDefinition_DtorBody` | ✅ | dtor | Stores the HashListMap<int,ClientActivityCategoryDefinition,128,-1> vtable at *this, then calls 00be52a0 on the head node at this+8 repeatedly until that head is null (no free, no flag test). |
| `00be6b40` | `HashListMapClientActivityCategoryDefinition_DtorBodyUnwind` | ✅ | dtor | Sets an SEH frame, stores the HashListMap<int,ClientActivityCategoryDefinition,128,-1> vtable at *this, calls 00be52a0 on the node at this+8 until the list is empty, then restores the frame and returns. |
