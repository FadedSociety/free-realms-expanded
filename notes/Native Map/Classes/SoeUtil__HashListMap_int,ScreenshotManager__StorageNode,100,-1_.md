# Class: SoeUtil::HashListMap<int,ScreenshotManager::StorageNode,100,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b4abc0` | `SoeUtil::HashListMap::Ctor` | ✅ | ctor | Constructs the screenshot storage hash map: stores SoeUtil::HashListMap<int,ScreenshotManager::StorageNode,100,-1>::vftable (018212c4), zeroes +4/+8/+0xc/+0x10 and memsets 400 bytes at +0x14, i.e. the 100 bucket heads (… |
| `00b4b2d0` | `SoeUtil::HashListMap::AllocNode` | ✅ | virtual | Node allocator: resolves the current allocator through FUN_006fb9c0/FUN_006fb860 and calls its virtual slot 1 when available, otherwise the global allocator at DAT_01bdf280 the same way, and finally FUN_006fb950(0x28,8)… |
| `00b4b310` | `SoeUtil::HashListMap::FreeNode` | ✅ | virtual | Node deallocator: calls FUN_00401c00(pointer,8) (accepted FreeMemory helper) with the argument supplied on the stack; the raw bytes are mov eax,[esp+4]; push 8; push eax; call; ret 4. |
| `00b4e3a0` | `— ⚠reject` | ⚪ | dtor | Eleven-byte function: stores the HashListMap vftable (018212c4) into [ECX] and tail-jumps to 00b4d0e0 with the same ECX. It is the cleanup entry reached from four EH unwind funclets. |
| `00b4e470` | `HashListMapScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, vtable slot 0 of HashListMap: stores the HashListMap vftable into [ECX], calls 00b4d0e0 on the object, and calls _free(this) when bit 0 of its stack byte is set; returns this. |
| `00b4e4a0` | `HashListMapDtorBody` | ✅ | dtor | Fastcall destructor body with an SEH frame: stores the HashListMap vftable into [ECX], then calls 00b4d0e0 with ECX = the object. It does not free the object. Called from two EH unwind funclets. |
| `00b4e730` | `HashListMapConstruct` | ✅ | ctor | Fastcall (ECX = storage map at ScreenshotManager+0x80). Stores the HashListMap vftable into [ECX], zeroes +4, +8, +0xc and +0x10, zeroes the 400-byte bucket array at +0x14 (100 entries) with _memset, and then stores the… |
