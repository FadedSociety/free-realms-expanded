# Class: GuiEditCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00634f23` | `— ⚠reject` | ⚪ | virtual | Vtable slot [127] of GuiEditCtrl (GuiPropEditCtrl has the same slot). Writes a width and height pair through param_2. Calls FUN_0041154d(&local, param_3); when that succeeds or param_3 is 4 it writes the stored pair wit… |
| `00635769` | `— ⚠reject` | ⚪ | virtual | Vtable slot [121] of GuiEditCtrl (GuiPropEditCtrl has the same slot). Calls FUN_00626cfe(this+0xf0, this+0xf4, 1). When the embedded object at +0x24 passes its vtable +0xa8 check it calls FUN_00612d3f(1) and this's virt… |
| `00635ca2` | `— ⚠reject` | ⚪ | virtual | Vtable slot [163] of GuiEditCtrl (GuiPropEditCtrl has the same slot). Returns 0 when no rect is passed and the embedded object at +0x24 fails vtable +0xa8. Otherwise it computes a visible-range rect (from the input rect… |
| `006373db` | `GuiEditCtrl::LayoutFormattedText ?` | 🟡 | virtual | Vtable slot [177] of GuiEditCtrl (GuiPropEditCtrl has the same slot). Builds a local GFormattedString-vtabled object, using the object at this+0x158 when set and a local default otherwise, and attaches a built text stor… |
| `006499a4` | `— ⚠reject` | ⚪ | virtual | Thiscall with this (param_1) and a source pointer (param_2). Returns at once when this+0x100 bit 0x40000 is set. Otherwise it calls vslot +8 of *param_2; a non-null result iterates its 16-byte elements (0048295a count, … |
| `0064a185` | `— ⚠reject` | ⚪ | virtual | Thiscall with a flag word (param_4) and object pointers (param_2, param_3, param_5); returns 1, or 0 on the guard exits. With flag 0x1000 set it clears the range pair this+0x10c/+0x110 and the child at this+0x120 and re… |
| `0064b71c` | `— ⚠reject` | ⚪ | virtual | Thiscall with an event record (param_2; 16-bit kind at record+0x1c, payload dwords at +0x4..+0x10) returning a byte. Compares the kind against about fifty DAT_ words in a compare chain; the kind values come from binary … |
