# Class: FirstTimeEventEnablePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c883e0` | `FirstTimeEventEnablePacketCtor` | ✅ | ctor | Constructs base first-time-event subtype 4, installs the enable-packet vtable, and stores the one-byte enabled flag. |
| `00c88750` | `FirstTimeEventEnablePacketScalarDeletingDtor` | ✅ | virtual | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
