# Class: SoeUtil::HashListMap<int,ItemSortDefinition,32,8>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `010272a0` | `HashListMapItemSort32x8Dtor` | ✅ | dtor | Destructor body for the 32,8 variant: stores its vtable, removes list heads through 01026e80, clears +0x9c and +0xa0, frees the node tree rooted at this+0x94 with alignment 4 (010270d0 on children, 00401c00 on the root)… |
| `01027d50` | `HashListMapItemSort32x8ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 010272a0 on this, then calls _free on this when bit 0 of the flag argument is set; returns this. |
| `01027d70` | `HashListMapItemSort32x8Ctor` | ✅ | ctor | Constructor: installs the 32,-1 vtable, clears +0x10, +0x8, +0xc, memsets 0x80 bytes at +0x14 (32 buckets), clears +4, then installs the 32,8 vtable and calls 01026bf0 on the sub-object at this+0x94 (lea ecx,[esi+0x94] … |
