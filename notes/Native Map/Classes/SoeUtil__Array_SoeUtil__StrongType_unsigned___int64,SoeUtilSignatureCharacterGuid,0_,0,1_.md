# Class: SoeUtil::Array<SoeUtil::StrongType<unsigned___int64,SoeUtilSignatureCharacterGuid,0>,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa9020` | `— ⚠reject` | ⚪ | virtual | Reads current capacity from this+0xc. With param_4 zero, grows a request above capacity to 1.25 times the request, keeps the existing buffer when current capacity is at most 4/3 of the request, and otherwise reallocates… |
| `00aa90c0` | `FreeAligned8 → CharacterGuidArrayFreeAlignedBuffer` | ✅ | virtual | Calls FUN_00401c00(param_1,8) to release the array buffer with alignment 8. |
| `00aa90e0` | `SoeUtil::Array::SwapTripleAndNotify` | ✅ | virtual | Virtual slot [3] of the SoeUtil::Array instantiation: exchanges the three dwords at this+4/+8/+0xc with param_2[1..3] using XOR swaps; when param_3 is non-zero it instead tail-dispatches to param_2's vtable slot +0xc wi… |
| `00aab9e0` | `SoeUtil::Array::~Array` | ✅ | dtor | Destructor of the array object: resets this+0 to the instantiation's vtable, clears the size at +8, frees the heap buffer at +4 with 8-byte alignment through FUN_00401c00 and clears the buffer word. |
| `00aabf80` | `SoeUtil::Array::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): resets the vtable, clears the size, frees the buffer at +4 with 8-byte alignment, clears the buffer and frees the object itself when the low bit of param_2 is set. |
| `00aac080` | `SoeUtil::Array::FieldInit04` | 🟢 | init | Initialises an embedded array member at object+4: stamps the instantiation's vtable at +4 and zeroes the buffer, size, capacity and one more dword (+8/+0xc/+0x10/+0x14). |
| `00aac0e0` | `— ⚠reject` | ⚪ | dtor | Destroys an embedded array member at object+4: resets the member vtable at +4, clears the capacity word at +0xc, frees the buffer at +8 with 8-byte alignment through FUN_00401c00 and clears the buffer. |
| `00ab2430` | `— ⚠reject` | ⚪ | function | Unlinks the first list node, repairs head, tail, and adjacent links, decrements the list count, releases its embedded CharacterGuid array storage, and frees the node. |
