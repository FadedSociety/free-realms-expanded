# Class: GComboBoxCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00606e9c` | `GComboBoxCtrl::SubProbeElseOff230` | 🟢 | virtual | Thiscall with three stack arguments (ret 0xc; A2 is read). When A2 is nonzero and the embedded object at this+0x24 accepts it (its vtable[0xd](A2) returns nonzero), returns. Otherwise calls this->vtable[0x8c](0,0) (slot… |
| `00606ec8` | `GComboBoxCtrl::ClearFlagEcRemoveId` | 🟢 | virtual | Thiscall with one 16-bit stack argument (ret 4). When it equals the short DAT_01bd5ea0, clears bit 0x10000000 in the dword at this+0xec and calls 00424fe0(DAT_01bd5ea0). |
| `00610454` | `GComboBoxCtrl::GetField118` | 🟢 | virtual | Thiscall without stack arguments: returns the dword at this+0x118. |
| `00610557` | `GComboBoxCtrl::GetField11c` | 🟢 | virtual | Thiscall without stack arguments: returns the dword at this+0x11c. |
| `00648f25` | `GComboBoxCtrl::ResetChildIfArgZero ?` | 🟡 | virtual | Thiscall with one byte stack argument (ret 4). When the argument is 0 and the dword at this+0x110 is non-null, pushes 1 and calls 00648dc8 on this; otherwise it returns with no effect. |
