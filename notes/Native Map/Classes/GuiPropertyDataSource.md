# Class: GuiPropertyDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00922a10` | `GuiPropertyDataSource_Ctor` | ✅ | ctor | Constructor: calls the GuiDsTable base constructor 00922890 on this with its name argument, stores the three GuiPropertyDataSource vtables at this, +0x30 and +0x34, and calls 008fe3d0 on the sub-object at this+0x254 bef… |
| `00922aa0` | `GuiPropertyDataSource_DtorBody` | 🟢 | dtor | Destructor body: calls 008ebca0 on the sub-object at this+0x254, then calls 0091bf60 on this. |
| `00922b00` | `GuiPropertyDataSource_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 00922aa0 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
