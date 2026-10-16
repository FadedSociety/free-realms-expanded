# Class: GuiProperties<GuiLabelCtrl>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00826e90` | `GuiProperties::Ctor` | 🟢 | ctor | Stores DAT_017efbd4 at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiLabelCtrl>::vftable at this+0, the embedded SoeUtil::Map<int,int,-1,0> vtable at +0x14, zeroes +0x18 and +0x1c. Returns nothing. |
| `00826f10` | `GuiProperties::ContainsEnumeratedValue ?` | 🟡 | virtual | Same body as 00825ca0 (count via slot +0x10, enumerate via slot +8, return 1 when the out value equals the argument, else 0). |
| `00826f70` | `GuiProperties::CallSlot6OnEnumeratedMatch ?` | 🟡 | virtual | Same body as 00825d00 (enumerate via slots +0x10 and +8; call slot +0x18 with the matched index). |
| `00827120` | `— ⚠reject` | ⚪ | virtual | Forwards its four arguments to 008270c0 and returns nothing (31 bytes). |
| `00827140` | `GuiProperties::GetNodeField8AtIndex ?` | 🟡 | virtual | Index and one out pointer: walks the global list at the GuiLabelCtrl head through +0x18 and stores node+8 at *out on success (returns 1, else 0). Same shape as 00826430. |
| `00827180` | `GuiProperties::EnumBeginGlobal ?` | 🟡 | virtual | Three stack out pointers. Sets cursor DAT_01be3a6c to the list head DAT_01be3ac4 and, when non-NULL, stores head+4, the key and the string-table entry to the outs and returns 1; otherwise 0. |
| `008271d0` | `GuiProperties::EnumNextGlobal ?` | 🟡 | virtual | Three stack out pointers. Advances the GuiLabelCtrl cursor through +0x18 and returns the entry as in EnumNextGlobal (1 while non-NULL, else 0). |
| `00827220` | `GuiProperties::GetGlobal34 ? ⚠rename` | 🟡 | virtual | No arguments. Returns a global dword (6 bytes: mov eax,[abs]; ret). |
| `00827230` | `GuiProperties::DtorBody` | 🟢 | dtor | Destructor body: stores GuiProperties<GuiLabelCtrl>::vftable at this+0, sets the embedded Map vtable at +0x14, tears the tree at +0x18 down with 008234e0 and the Map slot-3 free on the root when non-NULL, then zeroes +0… |
| `008272d0` | `—` | ⚪ | unknown | Virtual slot 11 of GuiProperties<GuiLabelCtrl>: same shape as the slot-11 routines 008267b0 and 00826d50 (table enumeration through the vtable, 00d518f0 and 00d51600 calls, Baked flag path). |
