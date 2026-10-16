# Class: MiniGamePayloadPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009b0960` | `MiniGamePayloadPacket::Ctor` | ✅ | ctor | Calls minigame base constructor with subtype 0xe and sentinels, installs payload packet vtable, and initializes byte array. |
| `009b09d0` | `MiniGamePayloadPacket::CtorWithPayload` | ✅ | ctor | Calls minigame base constructor with subtype 0xe and supplied param_2, installs packet vtable, initializes byte array. |
| `009b0a40` | `MiniGamePayloadPacket_ScalarDeletingDtor` | ✅ | dtor | Calls payload packet destructor and conditionally frees the object when deletion flag bit zero is set. |
