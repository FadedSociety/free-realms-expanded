# Class: SoeUtil::Array<GFxWrap::GuiGfxParam,3,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083ecb0` | `— ⚠reject` | ⚪ | virtual | __thiscall with one stack argument (buffer). If the buffer is not the inline storage at (this+0x17) & ~7, calls FreeMemory(buffer, 8). The inline storage is not freed. Same body as 0083ec90. |
| `00844710` | `ArrayGuiGfxParamCap3DtorBody` | 🟢 | dtor | __fastcall destructor body (157 bytes, SEH frame): stores the Array<GuiGfxParam,3,1> vtable, calls 00844110 (ArrayPopTailN) with the size at this+8, frees the buffer at this+4 with FreeMemory(buffer, 8) only when it is … |
| `008447b0` | `InlineOrAllocCap3` | 🟢 | virtual | Virtual slot 1 of Array<GuiGfxParam,3,1> (37 bytes, thiscall). When the count is above 3, passes the arguments to 0083eb60 (ArrayComputeCapacityAlloc). Otherwise writes 3 to the out-capacity and returns the inline stora… |
| `00844930` | `ArrayGuiGfxParamCap3Ctor` | 🟢 | ctor | __fastcall constructor (90 bytes): stores the Array<GuiGfxParam,0,1> vtable, zeroes this+4 (buffer), this+8 (size) and this+0xc (capacity), then stores the Array<GuiGfxParam,3,1> vtable and returns. |
| `008449b0` | `ArrayGuiGfxParamCap3ScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor (30 bytes): calls 00844710 (ArrayGuiGfxParamCap3DtorBody), then calls _free(this) when bit 0 of arg1 is set. Bytes after the _free call: add esp,4; mov eax,esi; pop esi; ret 4, so it … |
| `00844b00` | `— ⚠reject` | ⚪ | virtual | Virtual slot 3 of Array<GuiGfxParam,3,1> (241 bytes, same sequence as 00844a00): 00844660 with the other array's buffer and size, pop of the other array, 00844660 with this array's buffer and size, pop of this array, 00… |
