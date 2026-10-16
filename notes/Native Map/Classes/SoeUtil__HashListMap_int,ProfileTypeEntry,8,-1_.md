# Class: SoeUtil::HashListMap<int,ProfileTypeEntry,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a16240` | `HashListMap_ProfileTypeEntry_8_minus1_Dtor` | ✅ | dtor | Destructor body: installs the class vftable and then loops while the list head at this+8 is non-null, letting FUN_008bf2b0 remove and free one node at a time. |
| `00a163d0` | `HashListMap_ProfileTypeEntry_8_minus1_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0): installs the class vftable, drains the map by calling FUN_008bf2b0 on the head at this+8 until it is empty, then frees the object with _free when bit 0 of the flag argument is… |
