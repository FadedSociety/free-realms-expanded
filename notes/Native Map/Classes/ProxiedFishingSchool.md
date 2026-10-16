# Class: ProxiedFishingSchool

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd1210` | `ProxiedFishingSchool::ProxiedFishingSchool` | ✅ | ctor | Stores ProxiedFishingSchool::vftable at this+0; zeroes +0x8, +0x14, +0x18, +0x1c, +0x24, +0x28, +0x2c, +0x30, +0x34; stores SoeUtil::List<ProxiedFishingSchoolModelInfo,-1>::vftable at +0x10 and SoeUtil::List<ProxiedFish… |
| `00cd1550` | `— ⚠reject` | ⚪ | dtor | Stores ProxiedFishingSchool::vftable at this; walks the list headed at +0x14 calling virtual slot 2 of the object at +0x3c for each node; walks the list headed at +0x24 calling 007e7df0 for each node (once or twice depe… |
| `00cd16d0` | `ProxiedFishingSchool::ScalarDeletingDtor` | ✅ | dtor | Calls the ProxiedFishingSchool destructor body 00cd1550 on this, then frees this when bit 0 of the flag argument is set; returns this. |
