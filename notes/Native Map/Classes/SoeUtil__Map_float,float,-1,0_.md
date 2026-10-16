# Class: SoeUtil::Map<float,float,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00807d80` | `MapFloatFloatDtorBody` | 🟢 | dtor | Fastcall teardown with the same shape as MapFloatVec4DtorBody: stores the Map<float,float> vtable at this+0; if the block at this+4 is non-null calls 00803bc0 on its +0xc and +0x10 dwords, then calls this vtable slot +0… |
| `00808840` | `MapFloatFloatScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: stores the Map<float,float> vtable at this+0, calls 00803bc0 on the block's +0xc and +0x10 dwords and this vtable slot +0xc with the block when the block at this+4 is non-null, clears this+4… |
