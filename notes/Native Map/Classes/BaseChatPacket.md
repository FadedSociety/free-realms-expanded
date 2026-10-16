# Class: BaseChatPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b0d9a0` | `BaseChatPacket::Ctor` | ✅ | ctor | Installs BasePacket and BaseChatPacket vtables, sets packet type 15, and initializes the chat payload mode to zero. |
| `00b0da00` | `BaseChatPacket::CtorWithMode` | ✅ | ctor | Installs BasePacket and BaseChatPacket vtables, sets packet type 15, and stores the supplied mode in field 2. |
| `00b0da70` | `BaseChatPacket::Dtor` | ✅ | dtor | Restores BaseChatPacket and then BasePacket vtable pointers on the object. |
| `00b0dac0` | `BaseChatPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BaseChatPacket and BasePacket vtables and frees the object when the low flag bit is set. |
| `00b0dba0` | `BaseChatPacket::Dtor` | ✅ | dtor | Restores BaseChatPacket and BasePacket vtable pointers on the object. |
| `00b0f0c0` | `— ⚠reject` | ⚪ | dtor | Destroys two fixed 128-byte strings and two common subobjects, then restores BaseChatPacket and BasePacket vtables. |
| `00b0f1e0` | `BaseChatPacket::DtorWithString` | ✅ | dtor | Destroys a fixed 128-byte string and two common subobjects, then restores BaseChatPacket and BasePacket vtables. |
