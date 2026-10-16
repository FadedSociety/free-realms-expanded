# Class: SoeUtil::IStringFixed<unsigned_short,256>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a75920` | `IStringFixedWide256::Ctor` | ✅ | ctor | Installs the SoeUtil::IString<unsigned_short> vtable at this, points the buffer at +4 at the static empty buffer DAT_01be73a0, zeroes the count (+8) and capacity (+0xc), then installs the SoeUtil::IStringFixed<unsigned_… |
| `00a75980` | `IStringFixedWide256::ReserveBuffer` | ✅ | virtual | Virtual slot [1] of IStringFixed<unsigned short,256> (also slot 1 of the WideStringFixed<256> vtable). Takes the requested count (3 stack arguments, ret 0xc). When the count is at most 0x204 it stores 0 through the thir… |
| `00a759b0` | `IStringFixedWide256::FreeBufferIfNotInline` | ✅ | virtual | Virtual slot [2]. Takes a buffer pointer; when it is not this+0x10 (the inline buffer) it calls FreeMemory(pointer, 0); returns with ret 4. |
| `00a759d0` | `— ⚠reject` | ⚪ | dtor | Stores the IStringFixed<unsigned_short,256> vtable at this, then when the count at +0xc is positive releases the buffer through the refcount at buffer-4 and, on the last reference, vtable slot +8 of this (00a759b0, Free… |
| `00a75a90` | `IStringFixedWide256::ScalarDeletingDtor` | ✅ | dtor | Virtual slot [0]. Calls the dtor body 00a759d0 on this, then calls _free on this when bit 0 of its stack argument is set; returns this (ret 4). |
| `00a75b20` | `IStringFixedWide256::CleanupDtor` | 🟢 | method | Sets up an SEH frame, stores 0xffffffff as the state, calls 00a759d0 (the fixed wide-string destructor body) on this, then tears down the frame and returns. No value is returned. |
| `00c2d1b0` | `IStringFixed256Ctor` | ✅ | ctor | Constructor of the fixed wide-string type from a wide C string: installs SoeUtil::IString<unsigned_short>::vftable, points the inline buffer field at 0x01bed7c4, zeroes the length and count fields, then re-stamps the vf… |
| `00c2d220` | `IStringFixed256CopyCtor` | ✅ | ctor | Copy-style constructor of the fixed wide-string type: installs SoeUtil::IString<unsigned_short>::vftable, points the buffer field at 0x01bed7c4, zeroes the length/count fields, re-stamps SoeUtil::IStringFixed<unsigned_s… |
