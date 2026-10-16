# Class: SoeUtil::Map<SoeUtil::IString<char>,SoeUtil::Map<int,AssetManagement::Asset_const*,-1,0>,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `01094fa0` | `AllocNode0x2c` | 🟢 | virtual | Bytes: call FUN_006fb9c0; mov ecx,eax; call FUN_006fb860 (returns the thread-local manager or 0); if that is non-null calls manager->vtbl[1](0x2c,4) and returns its result; else if the global manager DAT_01bdf280 is non… |
| `01097630` | `MapStringToMapIntAssetDtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, ret 0. Stores the outer Map vtable at this; when the root at this+4 is non-null, runs the composite postorder destruction 01096260 on root+0x20 and root+0x24, calls the composite dest… |
| `01097680` | `MapStringToMapIntAssetScalarDeletingDtor` | ✅ | dtor | Thiscall with one stack arg (flags byte), returns this. Same teardown as 01097630 (outer vtable store, 01096260 on root children, 01095c20 and vtbl[3] on the root, zero this+4 and this+8); then if flags bit 0 is set cal… |
