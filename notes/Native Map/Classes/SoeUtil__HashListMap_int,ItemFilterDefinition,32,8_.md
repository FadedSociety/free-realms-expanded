# Class: SoeUtil::HashListMap<int,ItemFilterDefinition,32,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0107e2e0` | `HashListMapDerivedDtorBody` | 🟢 | dtor | Destructor body of the 32,8 variant: stores the HashListMap<int,ItemFilterDefinition,32,8> vftable, empties the list through 0107dce0, clears the tree root at +0x94 (children destroyed through 0107df30, root freed with … |
| `0107ee60` | `HashListMap32x8ScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: calls the destructor body 0107e2e0 on this, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
| `0107eea0` | `HashListMap32x8Ctor → HashListMap32x8CtorPoolAtPlus94` | 🟢 | ctor | Constructor body: stores the base HashListMap<int,ItemFilterDefinition,32,-1> vftable, zeroes this+0x10, +0x8 and +0xc, clears the 32-bucket array at this+0x14 (_memset 0x80 bytes), zeroes +0x4, stores the HashListMap<i… |
