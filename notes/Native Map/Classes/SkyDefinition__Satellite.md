# Class: SkyDefinition::Satellite

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00806930` | `SkyDefinition::Satellite::InitFromSource` | ✅ | ctor | Thiscall ctor. Stores SkyDefinition::TintedTexture::vftable, copies the dword at param_2+0x10 (AddRef on its +4 count) and the dwords at +0x20..+0x2c into this, then stores SkyDefinition::Satellite::vftable at this+0 an… |
| `00807a10` | `SkyDefinition::Satellite::ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: saves ECX to ESI, calls 00807490 (TintedTexture teardown body, DtorC) with this, then if bit 0 of the flag is set calls _free(this), and returns this in EAX (same byte shape as 008079f0 with… |
