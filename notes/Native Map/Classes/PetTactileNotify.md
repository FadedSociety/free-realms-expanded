# Class: PetTactileNotify

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00ccdb70` | `PetTactileNotify::Ctor` | ✅ | ctor | Initializes the pet packet base with subtype 22, installs PetTactileNotify vtable, and zeros four payload fields. |
| `00ccdf20` | `PetTactileNotify::ScalarDeletingDtor` | ✅ | dtor | Restores the BasePacket vtable and frees the packet allocation when the deleting flag is set. |
