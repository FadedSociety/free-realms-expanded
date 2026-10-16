# Class: SoeUtil::HashList<ScaleformConversionLayer::GFxKeyRecord,64,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083eb00` | `NodeAlloc0x1c` | 🟢 | virtual | Same shape as NodeAlloc0x18 but sizes 0x1c bytes, align 4: manager slot 1 via FUN_006fb860, else global manager slot 1, else FUN_006fb950(0x1c,4). Returns the block in EAX. |
| `0083eb40` | `NodeFree` | 🟢 | virtual | Stdcall-style single stack argument: calls FreeMemory(arg1, 4) then returns with ret 4. ECX is not used. |
| `008422b0` | `HashListScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor (30 bytes): calls 008414c0 (HashListMapDtorBodyEh, which stores the HashListMap vtable and removes nodes), then calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call… |
