# Class: FirstTimeEventScriptPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c88a00` | `FirstTimeEventScriptPacketCtor` | ✅ | ctor | Constructs base first-time-event subtype 5, initializes an owned string, and zero-initializes its integer list and other fields. |
| `00c88a80` | `FirstTimeEventScriptPacketDtor` | ✅ | dtor | Destroys the integer list, releases the owned string buffer, restores the BasePacket vtable, and returns. |
| `00c88da0` | `FirstTimeEventScriptPacketScalarDeletingDtor` | ✅ | virtual | Runs the script-packet destructor, frees the object when flag bit 0 is set, and otherwise returns it. |
