# Class: SoeUtil::JobHandler

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d4bb90` | `JobHandlerBaseDtor` | ✅ | dtor | Replaces the object's vtable with SoeUtil::JobHandler::vftable and returns without touching other fields. |
| `00d4bba0` | `JobHandlerScalarDeletingDtor` | ✅ | dtor | Installs the JobHandler vtable, conditionally frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
