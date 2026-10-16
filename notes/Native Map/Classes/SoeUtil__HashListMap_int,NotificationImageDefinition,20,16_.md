# Class: SoeUtil::HashListMap<int,NotificationImageDefinition,20,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01026400` | `HashListMapScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 01025980 on this, then calls _free on this when bit 0 of the flag argument is set; returns this. |
| `01026420` | `HashListMapCtor` | ✅ | ctor | Constructor: installs the HashListMap<...,-1> vtable, clears the fields at +4, +8, +0xc, +0x10 and memsets 0x50 bytes at +0x14 (20 buckets), installs the HashListMap<...,16> vtable, then calls 01025460 on the sub-object… |
