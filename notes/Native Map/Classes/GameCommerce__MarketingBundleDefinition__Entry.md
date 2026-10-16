# Class: GameCommerce::MarketingBundleDefinition::Entry

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b20a90` | `— ⚠reject` | ⚪ | ctor | SEH-framed constructor wrapper: sets an exception frame and calls FUN_00d5a8d0 with the exception-cookie chain as its only explicit argument; returns nothing. The callee is the function that stores GameCommerce::Marketi… |
| `00b20af0` | `— ⚠reject` | ⚪ | ctor | SEH-framed constructor wrapper that returns its first argument: calls FUN_00d5a990 (whose own vtables are GameCommerce::MarketingBundleDefinition::Entry plus SoeUtil::IStringNoShare<char>) and then returns param_1 uncha… |
| `00b25650` | `MarketingBundleDefinition_Entry::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: installs an SEH try level, calls FUN_00d5a8d0 with this in ECX (that body references GameCommerce::MarketingBundleDefinition::Entry::vftable and the SoeUtil::IString vtable, i.e. it destroys … |
