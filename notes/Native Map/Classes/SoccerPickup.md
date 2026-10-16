# Class: SoccerPickup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00cd6a30` | `— ⚠reject` | ⚪ | dtor | Restores the SoccerPickup vtable, releases its optional resource reference, and invokes slot 2 on the stored associated object when present. |
| `00cd6a70` | `SoccerPickup::SoccerPickup ?` | 🟡 | ctor | Stores the fourth argument at +4, installs the SoccerPickup vtable, clears +8 and +1c, stores the third argument at +0xc, and stores the second argument and its +4 field at +0x14 and +0x18. |
| `00cd6aa0` | `SoccerPickup::DeletingDestructor ?` | 🟡 | virtual | Calls the SoccerPickup destructor body, tests flag bit zero, conditionally frees the object, and returns the original object pointer. |
