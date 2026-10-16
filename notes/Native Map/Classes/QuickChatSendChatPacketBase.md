# Class: QuickChatSendChatPacketBase

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b90880` | `QuickChatSendChatPacketBase::Ctor` | ✅ | ctor | Calls the base quick-chat packet constructor, installs QuickChatSendChatPacketBase::vftable, initializes embedded fixed strings, and returns this. |
| `00b908e0` | `QuickChatSendChatPacketBase::CtorWithSubtype` | ✅ | ctor | Calls the base quick-chat constructor with the supplied subtype, installs QuickChatSendChatPacketBase::vftable, initializes embedded fixed strings, and returns this. |
| `00b90950` | `QuickChatSendChatPacketBase::DeletingDtor` | ✅ | dtor | Calls quick-chat base cleanup, frees this only when flag bit 0 is set, and returns this otherwise. |
