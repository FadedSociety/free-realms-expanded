# Class: TiXmlPrinter

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f9b70` | `TiXmlPrinter::Ctor` | ✅ | ctor | Installs TiXmlVisitor then TiXmlPrinter vtables, clears printer state and initializes two owned strings from global defaults. |
| `009f9c00` | `TiXmlPrinter::ScalarDeletingDtor` | ✅ | dtor | Calls the TiXmlVisitor cleanup, conditionally frees the object when flag bit 0 is set, and otherwise returns it. |
