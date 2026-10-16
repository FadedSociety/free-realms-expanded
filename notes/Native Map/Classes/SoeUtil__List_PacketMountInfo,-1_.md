# Class: SoeUtil::List<PacketMountInfo,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bcc20` | `AllocateListStorage` | ✅ | virtual | Initializes allocator state, then allocates 0x38 bytes aligned to eight through the thread manager, global manager, or aligned CRT fallback. |
| `008bcc60` | `FreeListStorage` | ✅ | virtual | Calls FUN_00401c00 with the supplied pointer and alignment 8. |
| `009197c0` | `SoeUtil::ListPacketMountInfoDtorBody` | ✅ | dtor | Destructor body, 39 bytes: stores the List vtable, then loops over the chain at +4 calling FUN_008fa8f0 as a thiscall on the container with the node pushed. |
| `0091a400` | `SoeUtil::ListPacketMountInfoScalarDtor` | ✅ | dtor | Scalar deleting destructor with the body inlined: stores the List vtable, loops over the +4 chain calling FUN_008fa8f0 (thiscall with the node pushed), then frees the object with _free when flag bit 0 is set. |
