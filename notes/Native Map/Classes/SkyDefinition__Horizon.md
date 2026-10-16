# Class: SkyDefinition::Horizon

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00806890` | `SkyDefinition::Horizon::InitFromSource` | ✅ | ctor | Thiscall ctor. Stores SkyDefinition::TintedTexture::vftable, copies the dword at param_2+0x10 (AddRef on its +4 count) and the dwords at +0x20..+0x2c into this, then stores SkyDefinition::Horizon::vftable at this+0 and … |
| `008079f0` | `SkyDefinition::Horizon::ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: saves ECX to ESI, calls 00807420 (TintedTexture teardown body, DtorB) with this, then if bit 0 of the flag is set calls _free(this), and returns this in EAX (bytes 56 8b f1 e8 .. f6 44 24 08… |
