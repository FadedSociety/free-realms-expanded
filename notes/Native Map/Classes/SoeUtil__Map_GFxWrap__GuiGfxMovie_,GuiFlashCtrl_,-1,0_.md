# Class: SoeUtil::Map<GFxWrap::GuiGfxMovie*,GuiFlashCtrl*,-1,0>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0083ea10` | `NodeAlloc0x18` | 🟢 | virtual | Allocates 0x18 bytes, align 4: calls thread memory manager slot 1 (manager from FUN_006fb860) with pushed (0x18,4), else global manager slot 1 via DAT_01bdf280, else FUN_006fb950(0x18,4). Returns the block pointer in EA… |
| `0083ea50` | `NodeFree` | 🟢 | virtual | Stdcall-style single stack argument: calls FreeMemory(arg1, 4) then returns with ret 4. ECX is not used. |
| `0083f460` | `MapDtorBody` | 🟢 | dtor | __fastcall: stores the Map vtable into this, then if this[1] is non-null calls FUN_0083df20 on node+0xc and node+0x10 and frees the node through vftable[3] (NodeFree); finally zeroes this[2] and this[1]. Does not free t… |
| `0083f4a0` | `MapScalarDeletingDtor` | ✅ | dtor | thiscall scalar deleting destructor: stores the Map vtable, releases the node at this[1] (0083df20 on +0xc and +0x10, then vftable[3] NodeFree), zeroes this[1] and this[2], and calls _free(this) when bit 0 of arg1 is se… |
