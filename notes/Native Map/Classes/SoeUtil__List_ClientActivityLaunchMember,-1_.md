# Class: SoeUtil::List<ClientActivityLaunchMember,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c4a3e0` | `List_Member_AllocateNode` | ✅ | virtual | Vtable slot 2 of the member list: the node allocator. It first calls 006fb9c0 (thread/allocator setup), then queries 006fb860 for an active allocator and, if present, invokes its virtual slot 1 (+4) to obtain the node; … |
| `00c4a420` | `List_Member_FreeNode` | ✅ | virtual | Vtable slot 3 of the member list: the node deallocator. Eighteen bytes (8b 44 24 04 6a 08 50 e8 .. c2 04 00) that call FreeMemory(node, 8) - the matching release for the 0x58-byte, 8-byte-aligned nodes handed out by 00c… |
| `00c4d020` | `ActivityMemberList_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0 of the member-list vtable): re-installs the list vtable, destroys every node with 00c4bb70 until the head is null, and frees the list object itself with CRT _free when bit 0 of … |
