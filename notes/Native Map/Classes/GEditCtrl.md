# Class: GEditCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00607f8c` | `GEditCtrl::ToggleFlag100AndNotify` | 🟢 | virtual | Thiscall with one 16-bit stack argument (ret 4). When it equals the short DAT_01bd5fd4, toggles bit 0x1 of the dword at this+0x100 and calls this->vtable[0xa9]() (slot 169). |
| `0060823b` | `GEditCtrl::TestArg1Zero` | 🟢 | virtual | Virtual with five stack arguments (ret 0x14). Returns AL=1 when the first stack argument is zero, else 0. ECX is not used. |
| `0061b838` | `GEditCtrl::ComputeLimitStyleWord ? ⚠downgrade` | 🟡 | virtual | Thiscall without stack arguments. iVar1 = 00607cd1 (returns this+0x190), iVar2 = 00607cf2 (returns this+0x194). When iVar1 is 0 it takes 00612c24(this) clamped to 0..0x32; when iVar2 is 0 it takes 00612be0(this) clamped… |
| `0061c532` | `GEditCtrl::ApplyStateByteArg` | 🟢 | virtual | Thiscall with one byte stack argument (ret 4). Calls 0041950b(A1). When A1 is zero: calls 00424fe0(DAT_01bd5fd4), clears bit 0x1 of this+0x100, calls this->vtable[0xa2]() (slot 162), zeroes this+0x294 and this+0x298, an… |
| `00621d93` | `GEditCtrl::BuildDrawState ? ⚠rename` | 🟡 | virtual | Thiscall with three stack arguments (Ghidra: param_2, param_3, param_4), returns int. Computes the control's width and height from this+0x94..0xa0 into a 0x7c-byte stack state, and when the mode checks pass builds GFill… |
