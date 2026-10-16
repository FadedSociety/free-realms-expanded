# Class: SoeUtil::Map<int,AssetManagement::Asset_const*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01095040` | `AllocNode0x18` | 🟢 | virtual | Same shape as 01094fa0 with size 0x18: FUN_006fb9c0, FUN_006fb860 thread-local manager lookup; manager->vtbl[1](0x18,4) when a manager is present, else the global manager DAT_01bdf280 vtbl[1](0x18,4), else FUN_006fb950(… |
| `01095a40` | `MapIntAssetDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, ret 0. Stores the Map<int, Asset const*> vtable at this; if the root at this+4 is non-null, runs FUN_01094ce0 on root+0xc and root+0x10 and calls this->vtbl[3](root); then sets this+8… |
| `01095a80` | `MapIntAssetScalarDeletingDtor` | ✅ | dtor | Thiscall with one stack arg (flags byte), returns this in EAX. Same teardown as 01095a40 (vtable store, postorder walk via 01094ce0, vtbl[3](root), zero this+4 and this+8); then if flags bit 0 is set calls _free(this). |
