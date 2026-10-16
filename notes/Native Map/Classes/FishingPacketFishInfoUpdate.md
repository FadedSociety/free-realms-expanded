# Class: FishingPacketFishInfoUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b69770` | `FishingPacketFishInfoUpdate::FishingPacketFishInfoUpdate` | ✅ | ctor | Calls the base fishing-packet constructor with subtype 4, installs the FishInfoUpdate vtable, and initializes its client-fish-entry list. |
| `00b6a570` | `ClearAndDeleteFishInfoUpdatePacket` | ✅ | dtor | Calls the FishInfoUpdate cleanup body and frees packet storage only when flag bit zero is set. |
