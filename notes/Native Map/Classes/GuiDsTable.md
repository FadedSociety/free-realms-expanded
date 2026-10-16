# Class: GuiDsTable

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00922890` | `GuiDsTable_Ctor` | ✅ | ctor | Base constructor: calls 00822990 with its name argument, stores the IDataAccessTable and GuiUserDataTable vtables at this+0x30/+0x34, zeroes +0x38, stores the GuiDsTable vtables at this, +0x30 and +0x34, sets the byte a… |
| `00922940` | `GuiDsTable_ScalarDtor` | 🟢 | dtor | Scalar deleting destructor: calls the body 0091bf60 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
