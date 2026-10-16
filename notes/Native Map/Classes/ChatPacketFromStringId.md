# Class: ChatPacketFromStringId

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0dbf0` | `ChatPacketFromStringId::Dtor` | ✅ | dtor | Restores ChatPacketFromStringId, BaseChatPacket, and BasePacket vtables in sequence. |
| `00b0dc50` | `ChatPacketFromStringId::ScalarDeletingDtor` | ✅ | dtor | Restores the concrete and base packet vtables and frees the object when the low flag bit is set. |
| `00b0e0f0` | `ChatPacketFromStringId::Ctor` | ✅ | ctor | Initializes BaseChatPacket mode 4, installs the concrete vtable, and initializes string and scalar payload fields from defaults. |
