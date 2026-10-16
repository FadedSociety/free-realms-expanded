# Class: SoeUtil::Array<GFxWrap::GuiGfxParam,0,1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083eb60` | `ArrayComputeCapacityAlloc` | 🟢 | virtual | stdcall, ret 0xc, ECX=this (capacity at this+0xc, buffer at this+4). Chooses a new capacity from requested count n and writes it to *arg3. With flag arg4 == 0: keeps capacity when n <= cap <= 4n/3, grows to 5n/4 when ca… |
| `0083ec00` | `— ⚠reject` | ⚪ | virtual | Stdcall-style single stack argument: calls FreeMemory(arg1, 8) and returns with ret 4. ECX is not used. |
| `0083ec20` | `ArraySwapOrDelegate` | 🟢 | virtual | thiscall with arg1 = other array and arg2 = flag. If flag != 0, tail-jumps to other->vftable[3] with (this, 0). If flag == 0, exchanges the buffer word at +4 and the words at +8 and +0xc between this and other (xor-swap… |
| `00844530` | `ArrayGuiGfxParamDtorBody` | 🟢 | dtor | __fastcall destructor body (41 bytes, no free of the object): stores the Array<GuiGfxParam,0,1> vtable, calls 00844110 (ArrayPopTailN) with the size at this+8 (removing every element), frees the buffer at this+4 with Fr… |
| `008447e0` | `ArrayGuiGfxParamScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor (61 bytes): stores the Array<GuiGfxParam,0,1> vtable, calls 00844110 (ArrayPopTailN) with the size, frees the buffer with FreeMemory(buffer, 8), clears this+4, then calls _free(this) … |
