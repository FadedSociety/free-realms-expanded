# Class: ChatPacketEnterArea

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0db30` | `ChatPacketEnterArea::Ctor` | ✅ | ctor | Initializes BaseChatPacket mode 2, installs the concrete vtable, and stores the argument in field 3. |
| `00b0e080` | `ChatPacketEnterArea::ScalarDeletingDtor` | ✅ | dtor | Restores BaseChatPacket and BasePacket vtables and frees the object when the low flag bit is set. |
