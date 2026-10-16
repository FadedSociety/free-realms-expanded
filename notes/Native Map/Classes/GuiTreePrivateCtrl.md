# Class: GuiTreePrivateCtrl

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00835c20` | `GuiTreePrivateCtrl::MatchShortAgainstGlobals ?` | 🟡 | virtual | Thiscall with one short argument, ret 0x1c. Returns AL=1 when param_2 equals the .data word DAT_01bc7488. Otherwise, when it equals DAT_01bc7244, it stores 0 at [this+0x1d8] and returns 1. Otherwise returns 0. |
| `008362c0` | `GuiTreePrivateCtrl::OnPointerDownUpMove` | 🟢 | virtual | Thiscall with five stack arguments, ret 0x14, void. If the message equals DAT_01bc7690 and [this+0x1d8] is 0 and (param_4 & 0x10000) is set, it sets [this+0x1d8]=1 and [this+0x1dc]=00664f7a(param_6-[this+0xf8]). If the … |
| `008363a0` | `GuiTreePrivateCtrl::ClassifyMessage158 ?` | 🟡 | virtual | Thiscall with eight stack arguments, ret 0x20. For messages DAT_01bc7308, DAT_01bc713c and DAT_01bc7488 it sets *param_9=1 when param_9 is non-null, and sets *param_8=3 when [this+0x60] has bit 0x800. For DAT_01bc7488 i… |
