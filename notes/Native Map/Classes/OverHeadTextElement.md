# Class: OverHeadTextElement

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0096f540` | `OverHeadTextElementCtor` | ✅ | ctor | Builds base overhead-element state from its arguments, installs the OverHeadTextElement vtable, stores two supplied values, sets fields to 0 and 0x14, and calls initialization helpers. |
| `0096f620` | `OverHeadTextElementDtor` | ✅ | dtor | Installs the OverHeadTextElement vtable, calls the text-element cleanup routine, then restores the OverHeadElement base vtable. |
| `0096f680` | `OverHeadTextElementScalarDeletingDtor` | ✅ | dtor | Calls the OverHeadTextElement destructor and frees object storage when the deleting flag's low bit is set. |
