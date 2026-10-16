# Class: BaseItemStatDefinition

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008acdd0` | `BaseItemStatDefinition_DtorBody ⚠downgrade` | 🟢 | dtor | Writes BaseItemStatDefinition::vftable at offset 0 and returns - the base-class destructor emission (no members released). Reached from four EH unwind funclets (0157eae0, 01581b00, 01581b90, 01583350) and listed as a de… |
| `008ade10` | `BaseItemStatDefinition_DtorBody_2` | 🟢 | dtor | Second identical emission of the base-class destructor body: writes BaseItemStatDefinition::vftable at offset 0 and returns. No callers recorded in the export. |
| `008c0480` | `BaseItemStatDefinition::ScalarDeletingDestructor` | ✅ | dtor | Installs BaseItemStatDefinition::vftable at object offset 0 and frees the object when deleting-flag bit 0 is set. |
