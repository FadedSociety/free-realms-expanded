# Class: SoeUtil::List<SoeUtil::StrongType<unsigned int,SoeUtilSignatureActorId,1>,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b63680` | `ListAllocateNode_0xC` | ✅ | virtual | Vtable slot 2 of the actor-signature-id list: recycles a node from the free lists or allocates FUN_006fb950(0xc,4) - 12 bytes with 4-byte alignment. |
| `00b636c0` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the actor-signature-id list: 18 bytes, bytes 8b4424046a0450e834e589ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b64870` | `ListActorId_InsertTail` | 🟢 | method | Allocates a node through the list's vtable slot 2, copies the caller's dword into node+0, writes the old tail to node+8 and 0 to node+4, links the old tail's +4 to the new node, promotes the node to head when the list w… |
| `00b65690` | `ListActorId_Clear` | 🟢 | method | Loop that repeatedly takes the head at this+4, unlinks the node using node+8 and node+4, decrements the count at this+0xc and hands the node to the list's free slot, until the list is empty. |
| `00b66520` | `ListActorId::~ListActorId` | ✅ | dtor | 11-byte tail-call destructor: stores the list vtable (mov dword[ecx],0x01822868) and jumps to FUN_00b65690, which frees every node. |
| `00b66660` | `ListActorId::~ListActorId` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, frees every node through FUN_00b65690, then frees the list object with _free when bit 0 of the flag argument is set. |
