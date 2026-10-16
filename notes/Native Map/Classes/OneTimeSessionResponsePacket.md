# Class: OneTimeSessionResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008ea370` | `OneTimeSessionResponsePacketCtor` | ✅ | ctor | Installs base and derived vtables, sets opcode 0x76, clears a flag, initializes embedded response and string member. |
| `008ea410` | `OneTimeSessionResponsePacketDtor` | ✅ | dtor | Decrements and conditionally destroys shared string buffer, cleans embedded response object and restores BasePacket vtable. |
| `008ea4c0` | `OneTimeSessionResponsePacket::ScalarDeletingDtor` | ✅ | dtor | Calls packet destructor, frees object if flag bit zero is set, otherwise returns original pointer. |
