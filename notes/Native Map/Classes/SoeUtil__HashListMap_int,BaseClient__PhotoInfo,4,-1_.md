# Class: SoeUtil::HashListMap<int,BaseClient::PhotoInfo,4,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bbc00` | `AllocateHashListStorage → SoeUtil::HashListMap<int,BaseClient::PhotoInfo,4,-1>:…` | ✅ | virtual | Initializes allocator state, then allocates 0x20 bytes aligned to four through the thread manager, global manager, or aligned CRT fallback. |
| `008bbc40` | `FreeHashListStorage → SoeUtil::HashListMap<int,BaseClient::PhotoInfo,4,-1>::Fre…` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 4. |
| `009105b0` | `BaseClientPhotoInfoMap::Cleanup` | ✅ | dtor | Restores the PhotoInfo hash-list-map vtable and repeatedly removes entries through 008e6e80 until the map is empty. |
| `009112f0` | `ScalarDeletingDtor → SoeUtil::HashListMapBaseClientPhotoInfo4ScalarDtor` | ✅ | dtor | Virtual at vftable[0] of 01801d38 (thiscall, one byte argument). Stores the HashListMap<int,BaseClient::PhotoInfo,4,-1> vftable at [this]; runs the clear loop (while [this+8] != 0 call FUN_008e6e80([this+8])); then if (… |
| `009156b0` | `DtorBodyClearHead2 → HashListMapPhotoInfo4DtorBody` | ✅ | dtor | __fastcall (this=ECX). Stores HashListMap<int,BaseClient::PhotoInfo,4,-1> vftable 0x01801d38 at [this]; calls FUN_008e6e80(this[2]) inside the clear loop (pseudocode shows FUN_008e6e80(param_1[2]) with the loop test on … |
