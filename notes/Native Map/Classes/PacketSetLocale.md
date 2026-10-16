# Class: PacketSetLocale

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ea580` | `PacketSetLocaleCtor ? ⚠downgrade` | 🟡 | ctor | Installs base and derived vtables, sets opcode 0x58 and initializes locale payload from argument. |
| `008ea5f0` | `PacketSetLocale::ScalarDeletingDtor` | ✅ | dtor | Calls locale destructor body, frees object if flag bit zero is set, otherwise returns original pointer. |
