# Class: SkyModelDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00807500` | `SkyModelDefinition::InitFromParent` | ✅ | ctor | Thiscall ctor, 3 stack args plus a char flag (param_4). Calls 0071b950 (RefCounted base init), copies the dword at param_2+0x1c8 into this+0xc, stores the AssetRequester vtable at this+8 and SkyModelDefinition::vftable … |
| `00807660` | `SkyModelDefinition::DtorBody` | 🟢 | dtor | Fastcall SEH-framed teardown: stores SkyModelDefinition::vftable at this+0 and this+8; if the object at this+0x14 exists it calls slot +0x3c of the object at this+0xc when its count is 1, decrements the count at +4 with… |
| `00807720` | `SkyModelDefinition::ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 00807660 (the SkyModelDefinition dtor body), then if bit 0 of the flag is set calls _free(this) (00d82463, thunk to 00d8a9d8), and returns this. |
