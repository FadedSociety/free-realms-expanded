# Class: SoeUtil::HashListMap<int,RailDefinition,64,32>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd890` | `RailDefinitionHashListMap32::InitBuckets` | 🟢 | init | Initializes bucket sentinels, advances a 32-bit pointer by 0x30 elements (0xc0 bytes) per step, and stores capacity 0x20. |
| `0091fe30` | `HashListMap_RailDefinition_Dtor` | ✅ | dtor | Destructor of the rail-definition hash map: writes the derived vtable, drains the node list at +8 through FUN_008fab60, frees the bucket arrays (FUN_008cb180 twice plus FUN_00401c00 on the header), then drains again aft… |
| `00920dd0` | `HashListMap_RailDefinition_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091fe30 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
| `00921130` | `HashListMap_RailDefinition_Ctor` | ✅ | ctor | Constructor: stores the HashListMap<int,RailDefinition,64,-1> vtable at this, zeroes +4, +8, +0xc and +0x10, memsets the 0x100-byte bucket array at +0x14, then stores the HashListMap<int,RailDefinition,64,32> vtable and… |
