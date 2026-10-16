# Class: BaseItemDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00723040` | `BaseItemDefinition::~BaseItemDefinition` | ✅ | dtor | Installs BaseItemDefinition::vftable, destroys the three embedded SoeUtil strings through 00722bf0 and finally rewrites the Loggable vtable of the embedded Loggable subobject at +0x10. |
| `007230d0` | `BaseItemDefinition_ScalarDeletingDtor` | 🟢 | dtor | Calls the BaseItemDefinition destructor body 00723040 for this and calls FUN_00401c00 (FreeMemory) when bit 0 of the flag byte is set. |
| `007230f0` | `BaseItemDefinition::BaseItemDefinition` | ✅ | ctor | Installs BaseItemDefinition::vftable, clears the leading fields, constructs the embedded IconData at +0x10 through 00720bf0(0,0), clears the trailing scalar fields, constructs three embedded IStringNoShare members throu… |
