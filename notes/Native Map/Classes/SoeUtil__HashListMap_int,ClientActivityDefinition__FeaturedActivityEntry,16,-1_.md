# Class: SoeUtil::HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01082450` | `HashListMapActivityCtor16` | 🟢 | ctor | Constructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable, zeroes +0x10, +0x8 and +0xc and +0x4, and clears the 16-bucket array at this+0x14 (_memset 0x40 bytes). |
| `01082780` | `HashListMapActivityDtorBody` | 🟢 | dtor | Destructor body: stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable and removes list nodes through 00be0a00 until the head at this+0x8 is null. Called from two EH unwind funclets. |
| `010827b0` | `HashListMapActivityScalarDeletingDtor` | 🟢 | dtor | Virtual slot 0 scalar deleting destructor: stores the vftable, removes list nodes through 00be0a00 until the head is null, then frees this with _free when flag bit 0 of the argument is set, and returns this. |
| `010827f0` | `HashListMapActivityDtorBodyEh` | 🟢 | dtor | Destructor body (EH variant): stores the HashListMap<int,ClientActivityDefinition::FeaturedActivityEntry,16,-1> vftable and removes list nodes through 00be0a00 until the head is null; the cookie argument in the call is … |
