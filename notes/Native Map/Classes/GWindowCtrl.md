# Class: GWindowCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0060c2c8` | `GWindowCtrl::TopBitsToCode` | 🟢 | virtual | Thiscall without stack arguments. Takes the top two bits of this+0xf0: 0 gives 0x4440033, 0x40000000 gives 0x8440033, 0x80000000 gives 0xc440033, and any other value (0xc0000000) gives 0x440033. |
| `0060c50d` | `GWindowCtrl::Notify0EPost7` | 🟢 | virtual | Thiscall with two unused stack arguments (ret 8). Calls 004396f7 with one pushed argument 0xe (ECX=this), then 00411506(7). |
| `0060c63e` | `GWindowCtrl::QueryPopupMenuProperty` | 🟢 | virtual | Thiscall without stack arguments. Builds the string "PopupMenu" (004e4e6d with length -1) in a temporary, calls the sub-object at this+0x24 through vtable[0x13](&temp) to obtain iVar1, releases the temporary through 004… |
| `0060c739` | `GWindowCtrl::ArgZeroCallSlot162` | 🟢 | virtual | Thiscall with one byte stack argument (ret 4). When the argument is zero, calls this->vtable[0xa2]() (slot 162) with no arguments. |
| `0061f417` | `GWindowCtrl::ForwardConst1` | 🟢 | virtual | Thiscall with one stack argument that is ignored (ret 4). Pushes the constant 1 and calls 0061ef45 (ECX=this from entry). |
| `00629b7a` | `GWindowCtrl::UpdateFrameChildren` | 🟢 | virtual | Vtable slot [120] of GWindowCtrl (GuiWindowCtrl has the same slot). Returns 0 at once when flag bit 0x10 at +0x90 is set or the owner pointer at +0x44 is null. Otherwise it sets bit 0x10 and, for four child names (Outer… |
| `00642e37` | `GWindowCtrl::StateModeDispatch ?` | 🟡 | virtual | Thiscall with an int mode (param_2) and three more int args. Handles modes 0, 1, 2, 3, 4 and 9; any other mode is forwarded to 0040f567. Mode 0 sends a zeroed record with DAT_01bd10ec through vslot 0x110. Mode 1 returns… |
