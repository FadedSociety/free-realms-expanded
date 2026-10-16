# Class: SkyConvolvedTexture

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008077e0` | `SkyConvolvedTexture::InitFromParent` | ✅ | ctor | Thiscall ctor with two stack arguments after this. Calls 0071b950, copies param_2+0x1c8 into this+0xc, stores the AssetRequester vtable at this+8, stores SkyTexture::vftable then SkyConvolvedTexture::vftable at this+0 a… |
| `00807950` | `SkyConvolvedTexture::ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: SEH-framed call to 00804800 (base teardown), then if bit 0 of the flag is set calls _free(this) (00d82463, thunk to 00d8a9d8), and returns this. |
