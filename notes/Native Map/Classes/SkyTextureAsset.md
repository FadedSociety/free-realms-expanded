# Class: SkyTextureAsset

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008095d0` | `SkyTextureAsset::InitWithHandle` | ✅ | ctor | Thiscall ctor with one stack argument (param_2): calls 0071bb80 (asset base constructor, accepted label AssetConstructor in another shard), stores SkyTextureAsset::vftable at this+0, stores param_2 at this+0x28 and clea… |
| `00809630` | `SkyTextureAsset::DtorBody` | 🟢 | dtor | Fastcall SEH-framed teardown: stores SkyTextureAsset::vftable at this+0, releases the refcounted object at this+0x2c (LOCK decrement, slot +4 at zero, pointer cleared), then calls 0071bc20 (asset base teardown, accepted… |
| `008096b0` | `SkyTextureAsset::ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 00809630 (SkyTextureAsset dtor body) with this, then calls _free(this) when bit 0 of the flag is set; returns this. |
