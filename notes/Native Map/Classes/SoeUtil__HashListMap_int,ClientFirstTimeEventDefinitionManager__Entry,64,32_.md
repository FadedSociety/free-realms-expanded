# Class: SoeUtil::HashListMap<int,ClientFirstTimeEventDefinitionManager::Entry,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009208e0` | `HashListMap_ClientFirstTimeEntry_Dtor` | ✅ | dtor | Destructor body: stores the HashListMap<int,ClientFirstTimeEventDefinitionManager::Entry,64,32> vtable, drains the node at this+8 through 008fac10, frees the two words at +8/+0xc of the pointer stored at this+0x114 thro… |
| `00920df0` | `HashListMap_ClientFirstTimeEntry_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 009208e0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `009211b0` | `HashListMap_ClientFirstTimeEntry_Ctor` | 🟢 | ctor | Constructor: stores the HashListMap<...,64,-1> vtable at this, zeroes the words at +4, +8, +0xc and +0x10, memsets the 0x100-byte bucket array at +0x14, then stores the HashListMap<int,ClientFirstTimeEventDefinitionMana… |
