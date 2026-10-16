# Class: FirstTimeEventStatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c88240` | `FirstTimeEventStatePacketCtor` | ✅ | ctor | Constructs base first-time-event subtype 2, installs the state-packet vtable, and stores the event id and one-byte state. |
| `00c88690` | `FirstTimeEventStatePacketScalarDeletingDtor` | ✅ | virtual | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
