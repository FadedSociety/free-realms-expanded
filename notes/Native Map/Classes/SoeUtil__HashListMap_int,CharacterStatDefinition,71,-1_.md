# Class: SoeUtil::HashListMap<int,CharacterStatDefinition,71,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102b240` | `HashListMapCtor71` | ✅ | ctor | Constructor: installs the HashListMap<int,CharacterStatDefinition,71,-1> vtable, clears +0x10, +0x8, +0xc, memsets 0x11c bytes at +0x14 (71 buckets), clears +4, and returns this. |
| `0102b700` | `HashListMapCharStatDtorBody` | ✅ | dtor | Destructor body: stores the HashListMap<int,CharacterStatDefinition,71,-1> vtable at offset 0, then drains the map through 0102b610. |
| `0102b7d0` | `HashListMapCharStatScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: stores the 71,-1 vtable, drains the map through 0102b610, then calls _free on this when bit 0 of the flag argument is set; returns this. |
| `0102b800` | `HashListMapCharStatDtorBody` | ✅ | dtor | Destructor body under an EH frame: stores the 71,-1 vtable at offset 0, keeps this in a local, then drains the map through 0102b610. |
| `0102b850` | `HashListMapCharStatDtorBody` | ✅ | dtor | Destructor body under an EH frame: stores the 71,-1 vtable at offset 0, keeps this in a local, then drains the map through 0102b610. |
| `0102bae0` | `HashListMap::Ctor` | ✅ | ctor | Writes the HashListMap<int,CharacterStatDefinition,71,-1> vftable, zeroes this+0x4, +0x8, +0xc, +0x10, memsets the 0x11c bytes at this+0x14 (71 bucket pointers) to 0 via _memset, then writes the HashList<CharacterStatDe… |
