# Class: SoeUtil::List<ItemRecord,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00b60` | `List_ItemRecord_AllocNode` | 🟢 | virtual | Allocator hook (bytes: call 006fb9c0; mov ecx,eax; call 006fb860; test eax,eax; push 4; push 0x18; je fallback; mov edx,[eax]; mov ecx,eax; mov eax,[edx+4]; call eax; ret): when the current thread memory manager exists … |
| `00a00ba0` | `List_ItemRecord_FreeNode` | 🟢 | virtual | Free wrapper: releases the node with 00401c00 at alignment 4 (same body as 00a006e0). |
| `00a03320` | `List_ItemRecord_0_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::List<ItemRecord,-1>: stores the class vtable, drains the nodes through 00a01d70 and frees this when the delete flag is set. |
