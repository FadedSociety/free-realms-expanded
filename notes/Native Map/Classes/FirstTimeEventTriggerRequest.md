# Class: FirstTimeEventTriggerRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c88180` | `FirstTimeEventTriggerRequestCtor` | ✅ | ctor | Constructs base first-time-event subtype 1, installs the trigger-request vtable, and stores the request value. |
| `00c88630` | `FirstTimeEventTriggerRequestScalarDeletingDtor` | ✅ | virtual | Restores the BasePacket vtable, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
