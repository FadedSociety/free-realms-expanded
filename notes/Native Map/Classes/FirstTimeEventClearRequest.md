# Class: FirstTimeEventClearRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c88320` | `FirstTimeEventClearRequestCtor` | ✅ | ctor | Constructs base first-time-event subtype 3, installs the clear-request vtable, and stores the event identifier. |
| `00c886f0` | `FirstTimeEventClearRequestScalarDeletingDtor` | ✅ | virtual | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
