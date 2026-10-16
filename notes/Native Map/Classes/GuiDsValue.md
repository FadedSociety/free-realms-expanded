# Class: GuiDsValue

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00821940` | `GuiDsValue::ReleaseOwned ?` | 🟡 | method | If (this+0x4c)&6 == 6: calls vtable slot 0 of the object at this+0x50 with argument 1, then clears bits 0x6 of this+0x4c and stores NULL at this+0x50. Returns nothing. |
| `00821990` | `GuiDsValue::ReadHeldPlus4 ?` | 🟡 | method | Returns the dword at [this+0x50]+4 when (this+0x4c)&6 == 6, otherwise returns 0 (bytes 8b 41 4c 83 e0 06 3c 06 75 07 8b 49 50 8b 41 04 c3 33 c0 c3). |
| `008219b0` | `GuiDsValue::~GuiDsValue` | 🟢 | dtor | Destructor body: stores GuiDsValue::vftable at this+0 and this+0x30, releases the owned object at +0x50 with the same logic as 00821940 (vtable slot 0 with arg 1, clear flag bits 0x6), then calls 00835610 and 00822810 a… |
| `00821b30` | `GuiDsValue::SetStringValue` | 🟢 | method | For a non-NULL string argument, when the flag argument is 0 or this+0x48 is 0: if (this+0x4c)&6 == 6 it calls 00738ae0 with the held string data pointer and the argument and returns when that call returns 0; otherwise i… |
| `00821ba0` | `GuiDsValue::RefreshFromSource` | 🟢 | virtual | Reads the source at this+0x18; if it dynamic-casts to GuiDsValue (00d8d382 with GuiDataSource and GuiDsValue descriptors) it copies the source value by its flag kind: 2 stores a dword at +0x20 and sets flag 2, 4 stores … |
| `00821d50` | `GuiDsValue::WriteValueText ?` | 🟡 | method | Reads flag kind at this+0x4c: 2 formats the int at +0x50 into the destination argument with DAT_01770978, 4 formats the float at +0x50 as a double with DAT_017e4014, 6 calls 006f9e60; always returns 1. |
