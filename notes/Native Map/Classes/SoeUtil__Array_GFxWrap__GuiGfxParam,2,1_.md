# Class: SoeUtil::Array<GFxWrap::GuiGfxParam,2,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083ec90` | `— ⚠reject` | ⚪ | virtual | __thiscall with one stack argument (buffer). If the buffer is not the inline storage at (this+0x17) & ~7, calls FreeMemory(buffer, 8). The inline storage is not freed. |
| `00844560` | `ArrayGuiGfxParamCap2DtorBody` | 🟢 | dtor | __fastcall destructor body (157 bytes, SEH frame): stores the Array<GuiGfxParam,2,1> vtable, calls 00844110 (ArrayPopTailN) with the size at this+8, frees the buffer at this+4 with FreeMemory(buffer, 8) only when it is … |
| `00844630` | `InlineOrAllocCap2` | 🟢 | virtual | Virtual slot 1 of Array<GuiGfxParam,2,1> (37 bytes, thiscall, args count and out-capacity pointer). When the count is above 2, passes the arguments to 0083eb60 (ArrayComputeCapacityAlloc). Otherwise writes 2 to the out-… |
| `008448d0` | `ArrayGuiGfxParamCap2Ctor` | 🟢 | ctor | __fastcall constructor (90 bytes): stores the Array<GuiGfxParam,0,1> vtable, zeroes this+4 (buffer), this+8 (size) and this+0xc (capacity), then stores the Array<GuiGfxParam,2,1> vtable and returns. |
| `00844990` | `ArrayGuiGfxParamCap2ScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor (30 bytes): calls 00844560 (ArrayGuiGfxParamCap2DtorBody), then calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call: add esp,4; mov eax,esi; pop esi; ret 4, so it … |
| `00844a00` | `ArrayExchangeCopyCap2` | 🟢 | virtual | Virtual slot 3 of Array<GuiGfxParam,2,1> (241 bytes, thiscall, arg1 = other). Calls 00844660 (ArrayAppendCopyN) with the other array's buffer and size, pops the other array (00844110 with its size), calls 00844660 with … |
