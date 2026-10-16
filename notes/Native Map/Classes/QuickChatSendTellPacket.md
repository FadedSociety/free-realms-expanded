# Class: QuickChatSendTellPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b90970` | `QuickChatSendTellPacket::Ctor` | ✅ | ctor | Calls the common constructor with subtype 2, installs QuickChatSendTellPacket::vftable, and initializes its embedded string vtable, empty-storage pointer, and two zero fields. |
| `00b909f0` | `QuickChatSendTellPacket::DeletingDtor` | ✅ | dtor | Calls tell cleanup, frees this only when flag bit 0 is set, and returns this otherwise. |
