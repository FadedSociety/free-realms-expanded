# Class: PacketMOTD

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ea4e0` | `PacketMOTDCtor` | ✅ | ctor | Installs base and PacketMOTD vtables, sets opcode 0x57 and initializes two embedded strings. |
| `008ea560` | `PacketMOTD::ScalarDeletingDtor` | ✅ | dtor | Calls MOTD destructor body, frees object if flag bit zero is set, otherwise returns original pointer. |
