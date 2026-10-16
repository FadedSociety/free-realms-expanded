# Class: Sky::LinearBlendController<SkyDefinition::TintedTexture>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00808040` | `LinearBlendController_TintedDtor` | 🟢 | dtor | Fastcall SEH-framed teardown: stores the Sky::LinearBlendController<SkyDefinition::TintedTexture> vtable at this+0, calls 00805890 (base teardown; receiver not shown by the pseudocode), then tears down three TintedTextu… |
| `008088e0` | `Sky::LinearBlendController_TintedScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 00808040 (the TintedTexture blend-controller dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns this (27-byte shape as 008079f0). |
