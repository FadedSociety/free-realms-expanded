# Class: SoeUtil::Array<SkyDefinition*,4,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00807df0` | `ArraySkyDefPtrDtorBody4` | 🟢 | dtor | Fastcall SEH-framed teardown: stores SoeUtil::Array<SkyDefinition*,4,1>::vftable at this+0, clears this+8, frees the buffer at this+4 with 00401c00(buffer,4) unless it is the inline area (this+0x13 aligned down to 4), c… |
| `00807eb0` | `ArraySkyDefPtrInlineOrAlloc4 ?` | 🟡 | virtual | Thiscall with a size and an out pointer (ret 0xc). If size <= 4 it stores 4 into *out and returns the inline buffer address (this+0x13 aligned down to 4); otherwise it tail-jumps to 00805680 with the size argument. |
| `00809f50` | `ArraySkyDefPtr4Ctor` | ✅ | ctor | Fastcall ctor (ECX = this): stores SoeUtil::Array<SkyDefinition*,0,1>::vftable at this+0, zeroes this+4, this+8 and this+0xc, then stores SoeUtil::Array<SkyDefinition*,4,1>::vftable at this+0 (the derived vtable overwri… |
| `0080a010` | `SoeUtilArraySkyDefPtr4ScalarDeletingDtor` | ✅ | dtor | Thiscall with one byte flag: calls 00807df0 (the Array<SkyDefinition*,4,1> teardown body) with this, then calls _free(this) when bit 0 of the flag is set; returns this. Same 27-byte shape as 008079f0. |
| `0080a610` | `DwordArrayMergeClear ? ⚠rename` | 🟡 | virtual | Thiscall with a second array pointer (param_2): calls 00807ee0 (DwordArrayAppend) with the data pointer and count of param_2 (zero when the count is below 1) and then clears param_2's count at +8; repeats the append wit… |
