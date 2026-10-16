# Class: InteractionDataSource

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c91f90` | `InteractionDataSource::ColumnName` | ✅ | virtual | Maps indices zero through eight to Id, Title, IconId, DefinitionId, Type, param1, param2, TooltipId, and Tooltip; invalid indices return an error label. |
| `00c921f0` | `InteractionDataSource::AdjustedRelease ? ⚠rename` | 🟡 | virtual | If the completion flag at this+0x23c is clear, calls the data-source reset helper, then calls FUN_010c8f30. |
| `00c92210` | `InteractionDataSource::AdjustedComplete ? ⚠rename` | 🟡 | virtual | If the completion flag at this+0x238 is clear, calls the data-source reset helper, then forwards param_2 to FUN_010c90c0. |
| `00c92700` | `InteractionDataSource::Ctor` | ✅ | ctor | Initializes the base GUI data-source object, installs InteractionDataSource vtables and its entry-array vtable, and stores supplied source values. |
| `00c927c0` | `InteractionDataSource::Dtor` | ✅ | dtor | Restores the embedded array vtable, destroys all interaction entries, frees the backing buffer, and destroys the embedded GUI user-data table. |
| `00c92850` | `InteractionDataSource::ScalarDeletingDtor` | ✅ | dtor | Calls InteractionDataSource destruction and frees the object only when the deleting flag's low bit is set. |
| `00c928a0` | `— ⚠reject` | ⚪ | virtual | Validates row access and returns the selected ID, strings, numeric values, parameters, tooltip ID, or tooltip text for columns zero through eight. |
