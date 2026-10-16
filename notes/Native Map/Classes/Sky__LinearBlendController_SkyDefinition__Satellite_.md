# Class: Sky::LinearBlendController<SkyDefinition::Satellite>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00808340` | `LinearBlendController_SatelliteCtor` | ✅ | ctor | Fastcall ctor (ECX = this): stores the Sky::LinearBlendController<SkyDefinition::Satellite> vtable at this+0, default-constructs three sub-objects by calling 00804bc0 with ECX = this+0x10, this+0x50 and this+0x90, store… |
| `008083d0` | `LinearBlendController_SatelliteDtor` | 🟢 | dtor | Fastcall SEH-framed teardown: stores the Sky::LinearBlendController<SkyDefinition::Satellite> vtable at this+0, calls 00805910 (base teardown), then calls 00807490 (TintedTexture-layout teardown body DtorC) three times … |
| `00808900` | `Sky::LinearBlendController_SatelliteScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 008083d0 (the Satellite blend-controller dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns this (27-byte shape as 008079f0). |
