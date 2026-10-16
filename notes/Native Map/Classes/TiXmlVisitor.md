# Class: TiXmlVisitor

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009f86b0` | `TiXmlVisitor::Dtor` | ✅ | dtor | Replaces the object's first dword with TiXmlVisitor::vftable and returns. |
| `009f8740` | `TiXmlVisitor::ScalarDeletingDtor` | ✅ | dtor | Restores TiXmlVisitor::vftable, frees the object when deleting flag bit 0 is set, and otherwise returns the object pointer. |
| `009f9170` | `TiXmlVisitor::Dtor ? ⚠rename` | 🟡 | dtor | Frees three owned buffers unless each is the shared empty buffer, then restores TiXmlVisitor::vftable. |
