# Class: SoeUtil::HashListMap<int,ClientClaimManager::ContentTracker,8,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00860` | `— ⚠reject` | ⚪ | virtual | Allocator hook (bytes: call 006fb9c0; mov ecx,eax; call 006fb860; test eax,eax; push 4; push 400; je fallback; mov edx,[eax]; mov ecx,eax; mov eax,[edx+4]; call eax; ret): when the current thread memory manager exists i… |
| `00a008a0` | `HashListMap_ContentTracker_FreeStorage` | 🟢 | virtual | Free wrapper: releases storage with 00401c00 at alignment 8 (same body as 00a007a0). |
| `00a032e0` | `HashListMap_ContentTracker_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of the tracker map: stores the HashListMap<int,ClientClaimManager::ContentTracker,8,-1> vtable, drains the map through 00a02150 and frees this when bit 0 of the flag is set. |
