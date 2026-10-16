# Class: GuiProperties<GuiImageCtrl>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00827410` | `GuiProperties::Ctor` | 🟢 | ctor | Stores DAT_017efbd4 at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiImageCtrl>::vftable at this+0, the embedded Map<int,int,-1,0> vtable at +0x14 with zero +0x18 and +0x1c. Returns nothing. |
| `00827490` | `GuiProperties::ContainsEnumeratedValue ?` | 🟡 | virtual | Same body as 00825ca0 (count via slot +0x10, enumerate via slot +8, return 1 on a matching out value, else 0). |
| `008274f0` | `GuiProperties::CallSlot6OnEnumeratedMatch ?` | 🟡 | virtual | Same body as 00825d00 (slot +0x18 call with the matched index). |
