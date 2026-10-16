# Class: SoeUtil::List<SoeUtil::Math::Vector<float,4>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008bd8e0` | `Vector4List::AllocateNode` | ✅ | virtual | Allocates a 0x20-byte node aligned to 0x10 after trying the thread and global allocators. |
| `008bd920` | `Vector4List::FreeNode` | ✅ | virtual | Releases the supplied node with 0x10-byte alignment. |
| `008e0d10` | `ListFloat4_Dtor` | ✅ | dtor | Stores the float4 list vtable and jumps to list cleanup. |
| `008e21e0` | `ListFloat4_ScalarDeletingDtor` | ✅ | dtor | Installs the float4 list vtable, runs list cleanup, and optionally frees the object when flag bit 0 is set. |
| `009ccbb0` | `SoeUtil::ListVector4::Dtor → SoeUtil::List<SoeUtil::Math::Vector<float,4>,32>::…` | ✅ | dtor | Restores the 32-node specialization vtable, drains linked nodes, frees tree nodes and resets counters, then restores the generic list vtable. |
