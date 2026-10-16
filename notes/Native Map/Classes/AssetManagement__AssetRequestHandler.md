# Class: AssetManagement::AssetRequestHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00927e80` | `AssetRequestHandler::Ctor` | ✅ | ctor | Initializes request/delivery vtables, embedded ready handler, two 32 KiB buffers, synchronization fields, and the requester pointer. |
| `00927f90` | `AssetRequestHandler::CallVirtual1cAsBool` | ✅ | virtual | Calls handler virtual slot 0x1c with the argument and returns whether its result is nonzero. |
| `00927fb0` | `AssetRequestHandler::SetStateByte` | ✅ | virtual | Stores the input byte at object offset 0x0c. |
| `00927fc0` | `AssetRequestHandler::GetStateByte` | ✅ | virtual | Returns the byte stored at object offset 0x0c. |
| `00927fd0` | `AssetRequestHandler::ScalarDeletingDtor` | ✅ | dtor | Runs helper 01098940 and frees the handler only when the deleting flag is set. |
| `00927ff0` | `AssetRequestHandler::DispatchByNonzeroValue` | ✅ | virtual | For nonzero mode calls 010957d0; for zero calls 01095850 with the first argument. |
| `00928030` | `AssetRequestHandler::ScalarDeletingDtor` | ✅ | dtor | Applies a four-byte this adjustment and jumps to the primary deleting-destructor implementation. |
| `00928040` | `AssetRequestHandler::GetBufferField359c` | ✅ | virtual | Returns the 32-bit field stored at object offset 0x359c. |
| `00928050` | `AssetRequestHandler::GetBufferFieldb5a4` | ✅ | virtual | Returns the 32-bit field stored at object offset 0xb5a4. |
| `01095e60` | `AssetRequestHandler::ReleaseAllEntries` | 🟢 | virtual | Fastcall (ECX=this), no stack args. Walks the list headed at this+0x135b0 (next at node+4): for each node releases the object at node+0 (refcount at object+4, vtbl[1] when it reaches zero). Then, while the head is non-n… |
| `01096830` | `AssetRequestHandler::InsertHashedEntry` | 🟢 | virtual | Thiscall (ECX=handler), one stack arg (node), ret 4. Computes a hash with FUN_00705b00 of the name copied from the receiver by FUN_0071c050 (which copies receiver+8, byte-verified; the second FUN_00705b00 argument is a … |
| `01096900` | `AssetRequestHandler::FindHashedEntryByName` | 🟢 | virtual | Thiscall (ECX=handler), one stack arg (key string pointer), ret 4, returns the node in EAX or 0. Hashes the key with FUN_00705b00 and walks the 8192-bucket table at this+0x35a0 + (hash & 0x1fff)*4 along node+0x20 for no… |
| `01096e00` | `— ⚠reject` | ⚪ | virtual | Fastcall-shaped thiscall (ECX=handler), one stack arg (ignored by the pseudocode), no return value. When the count at this+0xb5a4 is nonzero it walks the 8192-bucket table at this+0xb5a8 in index order; for each node it… |
| `01096f90` | `— ⚠reject` | ⚪ | virtual | Thiscall (ECX=handler), one stack arg (entry node), no return value (ret 4 by stack). When the byte at entry+0x1c is nonzero: if the entry's links +0x20 and +0x24 are both zero it first checks the entry is present in th… |
| `01097720` | `— ⚠reject` | ⚪ | virtual | Thiscall (ECX=handler), ret 0xc (three stack dwords by the ret immediate); the pseudocode shows the key pointer and callback object pointer. Looks the key up with vtbl[7] (+0x1c) and FUN_01096400, checks FUN_01095940 (H… |
| `01097910` | `— ⚠reject` | ⚪ | virtual | Thiscall (ECX=handler), three args as rendered (key object at arg2 +4 is the key string), no return value. Hashes the key string at arg+4 with FUN_00705b00, walks the chain at handler+0xb5a4 + (hash & 0x1fff)*4 for same… |
| `01098570` | `— ⚠reject` | ⚪ | virtual | Thiscall (ECX=handler), key, callback object and payload arguments (payload pointer and length as rendered), returns the result in EAX. Same key lookup and record append as SubmitKeyedRequest (01097720): vtbl[7] and FUN… |
| `010986f0` | `AssetRequestHandler::PeriodicTick` | 🟢 | virtual | Thiscall (ECX=handler), two stack args (decrement amount, budget), no return value. Calls the subobject at handler+0x38 through its vtbl[22], runs ListWalkDispatchWithTimeBudget (010983f0) with the budget; when that rep… |
| `01098750` | `Thunk_Adjust3c_010984f0` | 🟢 | virtual | Bytes 83 c1 3c (add ecx, 0x3c) then e9 (jmp 010984f0): an adjustor thunk that moves ECX to the subobject at handler+0x3c and jumps to ListFindByKeyDispatchRemove (010984f0); no body of its own. |
| `01098760` | `— ⚠reject` | ⚪ | virtual | Thiscall (ECX=subobject of handler, the 01805f08 base), two stack args (delivered object, key record whose string is at +4), no return value. Hashes the key string with FUN_00705b00 and walks the table at ECX+0xb5a4 + (… |
| `01098940` | `AssetRequestHandler::DtorBody` | ✅ | dtor | Fastcall (ECX=this), no stack args, ret 0. Stores the AssetRequestHandler vtable (01805f24) at this+0 and (01805f08) at this+4. When the count at this+0xb5a4 is nonzero walks the 8192-bucket table at this+0xb5a8, unlink… |
