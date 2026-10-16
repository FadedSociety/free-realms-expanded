# Class: FishingPacketRegisterPlayerResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b695f0` | `FishingPacketRegisterPlayerResponse::FishingPacketRegisterPlayerResponse` | ✅ | ctor | Calls the base fishing-packet constructor with subtype 3, installs the response vtable, and initializes its integer and fish-entry lists. |
| `00b6a550` | `ClearAndDeleteRegisterResponsePacket` | ✅ | dtor | Calls the registration-response cleanup body and frees packet storage only when flag bit zero is set. |
