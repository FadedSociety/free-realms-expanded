# Class: SoeUtil::List<FishScoringData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b63590` | `— ⚠reject` | ⚪ | method | Allocates a node through the list's vtable slot 2 (call at *param_1+8), writes the current tail into node+0x20 and 0 into node+0x1c, then links the node after the tail (old tail's +0x1c = new node), makes it the new tai… |
| `00b63610` | `ListAllocateNode_0x24` | ✅ | virtual | Vtable slot 2 of the fish scoring list: recycles a node from the free lists or allocates FUN_006fb950(0x24,4) - 36 bytes with 4-byte alignment. |
| `00b63650` | `ListFreeNode_4` | ✅ | virtual | Vtable slot 3 of the fish scoring list: 18 bytes, bytes 8b4424046a0450e8a4e589ff83c408c20400 = mov eax,[esp+4]; push 4; push eax; call FUN_00401c00; add esp,8; ret 4. |
| `00b65630` | `ListFishScoringData_Clear` | 🟢 | method | Loop that repeatedly takes the head at this+4, unlinks the node using node+0x20 and node+0x1c, decrements the count at this+0xc and hands the node to the list's free slot, until the list is empty. |
| `00b66510` | `ListFishScoringData::~ListFishScoringData` | ✅ | dtor | 11-byte tail-call destructor: stores the list vtable (mov dword[ecx],0x01822854) and jumps to FUN_00b65630, which frees every node. |
| `00b66630` | `ListFishScoringData::~ListFishScoringData` | ✅ | dtor | Scalar deleting destructor: stores the list vtable, frees every node through FUN_00b65630, then frees the list object with _free when bit 0 of the flag argument is set. |
