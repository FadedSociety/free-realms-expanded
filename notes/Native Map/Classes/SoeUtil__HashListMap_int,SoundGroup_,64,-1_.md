# Class: SoeUtil::HashListMap<int,SoundGroup*,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0081ba90` | `HashListMapSoundGroup64_DtorBody` | ✅ | dtor | Destructor body (does not free the object): restores SoeUtil::HashListMap<int,SoundGroup*,64,-1>::vftable at [0], then unlinks nodes one at a time through 00819ba0 (64-bucket removal, mask 0x3f) until the head at [2] is… |
| `0081be10` | `HashListMapSoundGroup64_DtorBodyUnwind` | ✅ | dtor | Destructor body called from EH unwind funclets (does not free the object): restores the HashListMap<int,SoundGroup*,64,-1> vtable at [0] and unlinks nodes through 00819ba0 until the head at [2] is null. Same body as 008… |
