# Class: SoeUtil::Array<unsigned_char,1024,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00805cb0` | `ArrayFreeIfOutOfLine ?` | 🟡 | virtual | Thiscall with ret 8: reads only the first stack argument p; if p is not this+0x10 (the inline buffer) it calls 00401c00(p, 1), FreeMemory with alignment 1. Second stack argument is never read. |
| `008086e0` | `ArrayU8_1024DtorBody` | 🟢 | dtor | Fastcall SEH-framed teardown: stores SoeUtil::Array<unsigned_char,1024,1>::vftable at this+0, clears this+8, frees the buffer at this+4 with 00401c00(buffer, 1) unless it is the inline area at this+0x10, clears this+4; … |
| `00808770` | `ArrayU8_1024InlineOrAlloc ?` | 🟡 | virtual | Thiscall with three stack arguments (size, out pointer, unused), ret 0xc (bytes 8b4424 04 3d00040000 7e09 ... e9 jmp 00702f80 ... c70000040000 8d4110 c20c00). If size > 0x400 it tail-jumps to 00702f80 (out-of-line alloc… |
| `00809fb0` | `ArrayU8_1024Ctor` | ✅ | ctor | Fastcall ctor (ECX = this): stores SoeUtil::Array<unsigned_char,0,1>::vftable at this+0, zeroes this+4, this+8 and this+0xc, then stores SoeUtil::Array<unsigned_char,1024,1>::vftable at this+0 (the derived vtable overwr… |
| `0080a030` | `SoeUtilArrayU8_1024ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 008086e0 (the Array<unsigned_char,1024,1> teardown body) with this, then calls _free(this) when bit 0 of the flag is set; returns this. Same 27-byte shape as 008079f0. |
