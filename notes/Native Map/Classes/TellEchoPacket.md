# Class: TellEchoPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0fa40` | `— ⚠reject` | ⚪ | ctor | Initializes BaseChatPacket mode 5, installs the concrete vtable, constructs chat strings, and initializes fixed text storage. |
| `00b0fab0` | `TellEchoPacket::ScalarDeletingDtor` | ✅ | dtor | Calls the string-bearing base chat destructor and frees the object when the low flag bit is set. |
