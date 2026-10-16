# Class: ObjectiveUIEventPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009afc10` | `ObjectiveUIEventPacket::Ctor` | ✅ | ctor | Calls objective base constructor with value 10, installs packet vtable, sets field +0x0c to -1, and initializes a string member. |
| `009afc80` | `ObjectiveUIEventPacket::Dtor` | 🟢 | dtor | Restores IString and BasePacket vtables, releases the owned string buffer, and returns. |
| `009b0ac0` | `ObjectiveUIEventPacket_ScalarDeletingDtor` | ✅ | dtor | Calls ObjectiveUIEventPacket destructor and conditionally frees object when deletion flag bit zero is set. |
