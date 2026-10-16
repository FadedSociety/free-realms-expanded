# Class: GuiImageCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00609138` | `GuiImageCtrl::FlagE8SelectCode` | 🟢 | virtual | Thiscall returning a byte: 0x44 when this+0xe8 has bit 0x10 set, otherwise 0x33 (test, neg, sbb, and 0x11, add 0x33). |
| `0061cfd4` | `GuiImageCtrl::NotifyFlagGatedCallouts` | 🟢 | virtual | Thiscall with one stack argument (ret 4). Calls 0041950b(A1). When A1 is zero and this+0xe8 has bit 0x10000000, calls 0061507b(0) with ECX=this. When A1 is nonzero and the 16-bit this+0xe8 has bit 0x1000, calls 00609254… |
| `0062745d` | `GuiImageCtrl::PaintBorderAndBlit` | 🟢 | virtual | Vtable slot [105] of GuiImageCtrl. Calls FUN_0040f51e and returns 2 when it returns 2. Otherwise it reads a width and height from this+0x94..0xa0 and the flag word at +0xe8. Inside the first flag/condition block it emit… |
| `0084ee80` | `GuiImageCtrl::TailForwardSlot107 → GuiImageCtrl::ForwardPointerEventToEnclosing…` | 🟢 | virtual | Calls 00406665 with ECX=this, loads the vtable of the returned object, and tail-jumps to its vtable slot +0x1ac (index 107) with ECX set to that object. No arguments of its own. |
