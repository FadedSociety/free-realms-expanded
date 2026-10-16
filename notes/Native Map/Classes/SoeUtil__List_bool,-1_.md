# Class: SoeUtil::List<bool,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b636e0` | `ListAllocateNode_0xC_bool` | ✅ | virtual | Vtable slot 2 of the bool list: recycles a node from the free lists or allocates FUN_006fb950(0xc,4) - 12 bytes with 4-byte alignment (one byte of payload plus the two links). |
| `00b63720` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the bool list: 18 bytes, bytes 8b4424046a0450e8d4e489ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b63740` | `— ⚠reject` | ⚪ | method | Allocates a node through the list's vtable slot 2, copies the caller's byte into node+0, writes the old tail to node+8 and 0 to node+4, links the old tail's +4 to the new node, makes the node the head when the list was … |
| `00b654e0` | `ListBool_Clear` | 🟢 | method | Loop that repeatedly takes the head at this+4, unlinks it using node+8 and node+4 (fixing this+4/this+8 and the neighbours), decrements the count at this+0xc, and hands the node to the list's free slot (vtable offset +0… |
| `00b66530` | `ListBool::~ListBool` | ✅ | dtor | 11-byte tail-call destructor: stores the list vtable (mov dword[ecx],0x0182287c) and jumps to FUN_00b654e0, which frees every node. |
| `00b66690` | `ListBool::~ListBool` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, frees every node through FUN_00b654e0, then frees the list object with _free when bit 0 of the flag argument is set. |
