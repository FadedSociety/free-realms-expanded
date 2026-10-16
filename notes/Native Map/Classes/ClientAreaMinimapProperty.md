# Class: ClientAreaMinimapProperty

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bed730` | `ClientAreaMinimapProperty::Ctor` | ✅ | ctor | Initializes ClientAreaProperty fields, stores constructor values and a float constant, installs the minimap vtable, and initializes two shared strings empty. |
| `00bed810` | `— ⚠reject` | ⚪ | virtual | Reads the property string through vtable slot +0x28, hashes up to 50 nonzero bytes with the 0x401 recurrence, and calls FUN_00705bc0 for long strings. |
| `00beddc0` | `ClientAreaMinimapProperty::TypeName ⚠downgrade` | 🟢 | virtual | Returns a pointer to the literal type identifier MiniMap. |
| `00beddd0` | `ClientAreaMinimapProperty::Dtor` | ✅ | dtor | Releases two reference-counted strings, invokes FUN_00ced950, and restores the base ClientAreaProperty vtable. |
| `00bede90` | `ClientAreaMinimapProperty::ScalarDeletingDtor` | ✅ | dtor | Calls the minimap property destructor, conditionally frees the object when flag bit 0 is set, and returns the object pointer otherwise. |
| `00ced8d0` | `ClientAreaMinimapProperty::WriteFields` | ✅ | virtual | Writes base property fields and the enterMinimapName and exitMinimapName values. |
| `00ced930` | `ClientAreaMinimapProperty::Enter` | ✅ | virtual | Calls the global client accessor, loads its field at offset 0x3c, obtains a nested context pointer, then tail-jumps to the minimap reset handler. |
| `00ced950` | `ClientAreaMinimapProperty::Exit` | ✅ | virtual | Calls the global client accessor, loads its field at offset 0x4c, obtains a nested context pointer, then tail-jumps to the minimap reset handler. |
| `00ced990` | `ClientAreaMinimapProperty::ReadFields` | ✅ | virtual | Reads enterMinimapName and exitMinimapName strings and assigns each to its matching property field. |
