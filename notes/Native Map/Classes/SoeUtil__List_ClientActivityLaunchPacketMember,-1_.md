# Class: SoeUtil::List<ClientActivityLaunchPacketMember,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c14e0` | `— ⚠reject` | ⚪ | virtual | Runs the container allocation hook, uses the thread or global allocator when available, and otherwise requests 0x50 bytes aligned to 8. |
| `009c1520` | `FreeActivityLaunchMemberNode` | ✅ | virtual | Calls the shared allocator free routine with the node pointer and alignment value 8. |
| `009c2e50` | `ActivityLaunchPacketMemberListDtor` | ✅ | dtor | Installs the member-list vtable and repeatedly invokes the node-removal helper until the list head is empty. |
| `009c2e80` | `ActivityLaunchPacketMemberListScalarDeletingDtor` | ✅ | dtor | Installs the member-list vtable, removes remaining nodes, then frees the allocation only when the deleting flag is set. |
