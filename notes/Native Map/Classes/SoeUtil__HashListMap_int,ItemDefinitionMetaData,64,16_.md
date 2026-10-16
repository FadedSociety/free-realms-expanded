# Class: SoeUtil::HashListMap<int,ItemDefinitionMetaData,64,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c731c0` | `HashListMap64Cap16::Dtor ? ⚠rename` | 🟡 | dtor | Under an EH frame stores the HashListMap<...,64,16> vtable, drains the head at +8 through 00c70910, frees the tree block at +0x114 (words [0x45]) through 00c71380 and 00401c00 (alignment 4), zeroes [+0x11c]/[+0x120], th… |
| `00c73d10` | `SoeUtil::HashListMap_int_ItemDefinitionMetaData_64_16::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body FUN_00c731c0 on this, then calls _free(this) when bit 0 of the flag argument is set; returns this. |
| `00c73dd0` | `— ⚠reject` | ⚪ | ctor | Constructor: stores the HashListMap<...,64,-1> then HashListMap<...,64,16> vtable; zeroes this+8, +0xc, +0x10 and +4; memsets 0x100 bytes (64 bucket heads) at this+0x14; calls FUN_00c706c0 with the cookie value in the e… |
