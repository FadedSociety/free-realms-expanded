# Class: SoeUtil::List<bool_(__thiscall_ClientClaimManager::*)(void)const_,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a006a0` | `List_Callback_AllocNode` | 🟢 | virtual | Allocator hook (bytes: call 006fb9c0; mov ecx,eax; call 006fb860; test eax,eax; push 4; push 0xc; je fallback; mov edx,[eax]; mov ecx,eax; mov eax,[edx+4]; call eax; ret): when the current thread memory manager exists i… |
| `00a006e0` | `List_Callback_FreeNode` | 🟢 | virtual | Free wrapper (18 bytes: mov eax,[esp+4]; push 4; push eax; call 00401c00; add esp,8; ret 4): releases the node with the engine FreeMemory helper at alignment 4. |
| `00a02400` | `List_Callback_0_ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor of SoeUtil::List<bool (ClientClaimManager::*)(void)const,-1>: stores the class vtable, drains the list through 00a01370 and frees this when bit 0 of the flag is set. |
