# Class: SoeUtil::HashListMap<int,ItemDefinitionMetaData,4096,1024>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c72f20` | `HashListMap4096Cap1024::Dtor ? ⚠rename` | 🟡 | dtor | Under an EH frame stores the HashListMap<...,4096,1024> vtable, drains the head at +8 through 00c70520, frees the tree block at +0x4014 through 008cadb0 and 00401c00 (alignment 4), zeroes +0x401c and +0x4020, then zeroe… |
| `00c73cd0` | `SoeUtil::HashListMap_int_ItemDefinitionMetaData_4096_1024::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c72f20 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c73d50` | `— ⚠reject` | ⚪ | ctor | Constructor: stores the HashListMap<...,4096,-1> then HashListMap<...,4096,1024> vtable; zeroes this+8, +0xc, +0x10 and +4; memsets 0x4000 bytes (4096 bucket heads) at this+0x14; calls security-cookie check FUN_008bc460. |
