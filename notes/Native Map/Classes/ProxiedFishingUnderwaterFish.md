# Class: ProxiedFishingUnderwaterFish

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd1760` | `ProxiedFishingUnderwaterFish::ProxiedFishingUnderwaterFish` | ✅ | ctor | Stores ProxiedFishingUnderwaterFish::vftable at this; sets the dword at +0x20 to 1 and the dword at +0x150 to 1; stores arg2 at +0x170; zeroes most other fields; stores two results of 00738380 at +0xc4 and +0xd0; copies… |
| `00cd1950` | `ProxiedFishingUnderwaterFish::~ProxiedFishingUnderwaterFish` | ✅ | dtor | Stores ProxiedFishingUnderwaterFish::vftable at this; when the dword at +0x15c is non-zero, calls 007dc240 and 008c52a0 on it and 009c36a0 on the result, then clears +0x15c; when the dword at +0x18 is non-zero, calls vi… |
| `00cd19f0` | `ProxiedFishingUnderwaterFish::ScalarDeletingDtor` | ✅ | dtor | Calls destructor body 00cd1950 on this, then frees this when bit 0 of the flag argument is set; returns this. |
