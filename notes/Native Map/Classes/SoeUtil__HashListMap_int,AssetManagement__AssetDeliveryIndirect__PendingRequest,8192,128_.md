# Class: SoeUtil::HashListMap<int,AssetManagement::AssetDeliveryIndirect::PendingRequest,8192,128>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0109ca00` | `HashListMap::~HashListMap8192_128` | ✅ | dtor | SEH-framed destructor body of the 128-variant pending-request map: installs the 128 vtable, drains the node list through 0109b9f0, releases the auxiliary block at +0x8014 (010996d0 on its two words, then 00401c00), then… |
| `0109d490` | `Thunk_0109d3b0_Adjust8014 ?` | 🟡 | function | Adjustor thunk (bytes 81 c1 14 80 00 00 then jmp rel32): adds 0x8014 to ECX and jumps to 0109d3b0. |
| `0109d4a0` | `Thunk_0109d420_Adjust8014 ?` | 🟡 | function | Adjustor thunk (bytes 81 c1 14 80 00 00 then jmp rel32): adds 0x8014 to ECX and jumps to 0109d420. |
| `0109d530` | `HashListMap::ScalarDeletingDestructor128` | ✅ | dtor | Scalar deleting destructor shape: calls the 128-variant pending-request map body 0109ca00, then calls _free on the object when bit 0 of the stack argument is set; returns the object pointer. |
| `0109d570` | `HashListMap::HashListMap8192_128` | ✅ | ctor | Constructor: installs the -1 map vtable, zeroes the words at +0x10, +8 and +0xc, clears the 0x8000-byte bucket table at +0x14 with _memset, zeroes +4, installs the 128-variant vtable and runs the base initializer 010991… |
