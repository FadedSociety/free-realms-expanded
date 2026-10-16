# Class: SoeUtil::List<DefinitionEntry,16>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b08c00` | `InlineDefinitionEntryListDestructor` | 🟢 | dtor | Sets the inline-list vtable, removes entries, clears/frees its embedded tree, restores the base list vtable, and cleans remaining entries. |
| `00b08cd0` | `ScalarDeletingDtor` | 🟢 | dtor | Runs inline DefinitionEntry list destruction and optionally frees the containing object. |
