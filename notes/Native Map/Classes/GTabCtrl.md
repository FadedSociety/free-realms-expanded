# Class: GTabCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00653906` | `GTabCtrl::StyleCodeFromFlags ?` | 🟡 | virtual | No stack arguments; this is in ECX. Reads the dword at this+0xf0 and returns a 32-bit code in EAX. If (flags & 0x300) is 0 it returns 0x220044. Otherwise it returns 0x220014, 0x220024, 0x220041 or 0x220042 for (flags & … |
| `006539f7` | `GTabCtrl::ConditionalStateReset ?` | 🟡 | virtual | Thiscall with one byte stack argument (ret 4). When the argument is nonzero, it calls 006538c3 on this with 0 if either this+0xf4 bit 0x10000000 is clear, or the owner returned by 0040f8df differs from this and (this+0x… |
| `00660f32` | `— ⚠reject` | ⚪ | virtual | thiscall(this = tab control state, param_2 = property sink). Returns 0 when 00418ce1(sink) returns false. Otherwise: TabFlags (value string GFC_CF_TAB_LEFT, values +0xe8 and +0xec) via sink vtbl+0x48; CurrentItemIndex (… |
| `00661037` | `GTabCtrl::AccumulateTotalSlot65` | 🟢 | virtual | fastcall, this in ECX. Starts from 0040eb1d(). When this+0xfc is non-null and its vtbl+0x3c() returns this: adds the held child's value (004c6dcb(&this,0), 004cd10a(), held vtbl+8 result; then either child->vtbl+0x3c(0,… |
| `0066ca4d` | `—` | ⚪ | unknown | Virtual slot [120] shared by GTabCtrl and GuiTabCtrl (both vtable slots hold 0066ca4d). Body (16505 bytes of pseudocode) was not read in this pass. |
| `006740a1` | `—` | ⚪ | unknown | Virtual slot [105] shared by GTabCtrl and GuiTabCtrl (both vtable slots hold 006740a1). Body (9396 bytes of pseudocode) was not read in this pass. |
| `006a84ea` | `— ⚠reject` | ⚪ | method | Thiscall with one char argument. If the argument is 0 and bit 0x8000000 of [this+0xf4] is set, it calls 0067b53e and 006a21ac, then clears that bit. If the argument is nonzero and the bit is clear, it calls 006a6233(thi… |
| `006a8d9e` | `—` | ⚪ | unknown | Virtual slot [127] shared by GTabCtrl and GuiTabCtrl (both vtable slots hold 006a8d9e). Body (7042 bytes of pseudocode) was not read in this pass. |
