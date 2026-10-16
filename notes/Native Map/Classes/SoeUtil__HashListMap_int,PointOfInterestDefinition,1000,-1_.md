# Class: SoeUtil::HashListMap<int,PointOfInterestDefinition,1000,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0102fa80` | `HashListMap::Ctor → PointOfInterestDefinitionHashListMap::Ctor` | ✅ | ctor | Fastcall constructor body (ECX = table): stores the HashListMap<int,PointOfInterestDefinition,1000,-1> vftable, zeroes +0x4, +0x8, +0xc, +0x10, and memsets the 4000 bytes at +0x14 (1000 bucket heads) through _memset. |
| `0102fba0` | `HashListMap::AllocNode100 → SoeUtil::HashListMap<int,PointOfInterestDefinition,…` | ✅ | virtual | Virtual with no stack args: calls 006fb9c0, gets the current thread memory manager (006fb860); if present pushes (100,4) and calls its slot [1]; else global manager slot [1] (DAT_01bdf280); else calls 006fb950(100,4). |
| `0102fbe0` | `HashListMap::FreeNode → SoeUtil::HashListMap<int,PointOfInterestDefinition,1000…` | ✅ | virtual | Thiscall virtual with one stack argument (ret 4; bytes 8b 44 24 04 6a 04 50 e8 ...): calls FreeMemory 00401c00 on the argument with alignment 4 and returns. |
| `010303a0` | `HashListMap::DtorBody → SoeUtil::HashListMap<int,PointOfInterestDefinition,1000…` | ✅ | dtor | Fastcall destructor body (ECX = table): stores the HashListMap<int,PointOfInterestDefinition,1000,-1> vftable and removes nodes through 00b98160 while the first node is non-null; no free. |
| `010303d0` | `— → SoeUtil::HashListMap<int,PointOfInterestDefinition,1000,-1>::DeletingDtor ⚠…` | ⚪ | dtor | Scalar deleting destructor (thiscall, flag byte): stores the HashListMap<...,1000> vftable, removes rows through 00b98160 while the first node is non-null, and calls _free(this) when bit 0 of the flag is set; returns th… |
