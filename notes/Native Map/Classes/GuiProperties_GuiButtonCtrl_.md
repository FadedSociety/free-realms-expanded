# Class: GuiProperties<GuiButtonCtrl>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00826910` | `GuiProperties::Ctor` | 🟢 | ctor | Stores the value DAT_017efbd4 (dword) at this+4 and +8, zeroes +0xc and +0x10, stores GuiProperties<GuiButtonCtrl>::vftable at this+0, stores SoeUtil::Map<int,int,-1,0>::vftable at +0x14 with zero root (+0x18) and count… |
| `00826990` | `GuiProperties::ContainsEnumeratedValue ?` | 🟡 | virtual | thiscall with one int argument: reads a count from vtable slot +0x10 of this, then for each index calls vtable slot +8 with (index, &local, &out) and returns 1 when the out value equals the argument; returns 0 when no i… |
| `008269f0` | `GuiProperties::CallSlot6OnEnumeratedMatch ?` | 🟡 | virtual | thiscall with one int argument: reads a count from vtable slot +0x10, enumerates indices calling vtable slot +8 with (index, &local, &out); at the first index whose out value equals the argument it calls vtable slot +0x… |
| `00826ba0` | `— ⚠reject` | ⚪ | virtual | Forwards its four arguments to 00826b40 (indexed list read) and returns nothing (31 bytes). |
| `00826bc0` | `GuiProperties::GetNodeField8AtIndex ?` | 🟡 | virtual | Index and one out pointer: walks the global list at DAT_01be2c84 through +0x18 for index steps; when a node is reached stores node+8 at *out and returns 1, else 0. |
| `00826c00` | `GuiProperties::EnumBeginGlobal ?` | 🟡 | virtual | Three stack out pointers. Sets cursor DAT_01be2c2c to the list head DAT_01be2c84 and, when non-NULL, stores head+4, the key dword and the string-table entry (&PTR_DAT_01b34f88)[key*4] to the outs and returns 1; otherwis… |
| `00826c50` | `GuiProperties::EnumNextGlobal ?` | 🟡 | virtual | Three stack out pointers. Advances cursor DAT_01be2c2c through +0x18; when still non-NULL stores its +4, key and string-table entry to the outs and returns 1; otherwise 0. Same body as 008264c0 with the ButtonCtrl curso… |
| `00826ca0` | `GuiProperties::GetGlobal8c ?` | 🟡 | virtual | No arguments. Returns the dword DAT_01be2c8c (6 bytes: mov eax,[abs]; ret). |
| `00826cb0` | `GuiProperties::DtorBody` | 🟢 | dtor | Destructor body for GuiProperties<GuiButtonCtrl>: stores its vtable at this+0, sets the embedded Map vtable at +0x14, tears the tree at +0x18 down with the post-order walk and the Map slot-3 free, then zeroes +0x1c and … |
| `00826d50` | `—` | ⚪ | unknown | Virtual slot 11 of GuiProperties<GuiButtonCtrl>: same shape as the GuiProperties<GuiEditCtrl> slot-11 routine 008267b0 (enumerates through the vtable and writes the entries with 00d518f0 and 00d51600), with the Button t… |
