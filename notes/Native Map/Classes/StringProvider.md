# Class: StringProvider

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089c470` | `StringProvider::~StringProvider` | ✅ | dtor | Trivial destructor body: writes the StringProvider vtable pointer at +0 and returns; reached only from EH unwind funclets. |
| `0089c480` | `StringProviderScalarDtor` | ✅ | dtor | Scalar deleting destructor of StringProvider: stores the class vtable at +0 and calls _free when bit 0 of param_2 is set. |
