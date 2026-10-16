# Class: OverHeadHpBarElement

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00957a40` | `OverHeadHpBarElementCtor ?` | 🟡 | ctor | Calls the OverHeadElement base constructor, installs the HP-bar vtable, stores one supplied value, and initializes two fields including capacity 0x10. |
| `00957af0` | `OverHeadHpBarElementDtor ?` | 🟡 | dtor | Restores the derived HP-bar vtable, then the OverHeadElement base vtable. |
| `00957b60` | `OverHeadHpBarElement::ScalarDeletingDtor ?` | 🟡 | dtor | Restores derived then base vtables, and frees the object when deletion flag bit 0 is set. |
