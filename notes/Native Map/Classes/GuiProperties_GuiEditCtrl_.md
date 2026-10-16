# Class: GuiProperties<GuiEditCtrl>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00825bf0` | `GuiProperties::Ctor` | 🟢 | ctor | Stores GuiProperties<GuiEditCtrl>::vftable at this+0, sets this+4 and +8 to the dword DAT_017efbd4, zeroes +0xc and +0x10, stores SoeUtil::Map<int,int,-1,0>::vftable at +0x14 (an embedded Map), zeroes its root at +0x18 … |
| `00825ca0` | `GuiProperties::ContainsEnumeratedValue ?` | 🟡 | virtual | thiscall with one int argument: reads a count from vtable slot +0x10 of this, then for each index calls vtable slot +8 with (index, &local, &out) and returns 1 when the out value equals the argument; returns 0 when no i… |
| `00825d00` | `GuiProperties::CallSlot6OnEnumeratedMatch ?` | 🟡 | virtual | thiscall with one int argument: reads a count from vtable slot +0x10, enumerates indices calling vtable slot +8 with (index, &local, &out); at the first index whose out value equals the argument it calls vtable slot +0x… |
| `00826410` | `— ⚠reject` | ⚪ | virtual | Four stack arguments forwarded unchanged to 00826070 (indexed list read); returns nothing (31 bytes). |
| `00826430` | `GuiProperties::GetNodeField8AtIndex ?` | 🟡 | virtual | One stack argument (out pointer) and an index (param_1): walks the global list at DAT_01be3d2c through the next link at +0x18 for index steps; when a node is reached stores node+8 at *out and returns 1, else returns 0. |
| `00826470` | `GuiProperties::EnumBeginGlobal ?` | 🟡 | virtual | Three stack out pointers. Sets the cursor DAT_01be3ccc to the global list head DAT_01be3d2c; when the head is non-NULL stores head+4 (third out), the key dword head+0 (second out) and the string-table entry (&PTR_DAT_01… |
| `008264c0` | `GuiProperties::EnumNextGlobal ?` | 🟡 | virtual | Three stack out pointers. Advances the cursor DAT_01be3ccc through its +0x18 link; when it is still non-NULL stores its +4 value, key dword +0 and string-table entry (&PTR_DAT_01b34f88)[key*4] to the outs and returns 1;… |
| `00826510` | `GuiProperties::GetGlobal34 ?` | 🟡 | virtual | No arguments. Returns the dword DAT_01be3d34 (6 bytes: mov eax,[abs]; ret). |
| `00826710` | `GuiProperties::DtorBody` | 🟢 | dtor | Stores GuiProperties<GuiEditCtrl>::vftable at this+0, sets the embedded Map vtable SoeUtil::Map<int,int,-1,0> at +0x14, tears down the tree at this+0x18 through 008234e0 (children) and the Map's vtable slot 3 (00824990)… |
| `008267b0` | `—` | ⚪ | unknown | Virtual slot 11 of GuiProperties<GuiEditCtrl>: calls FUN_00d52e70 and then loops with the vtable entries +0 and +0x74 and +4 (enumeration), calls 00d518f0 per entry with the entry's +4 field and a local string, appends … |
