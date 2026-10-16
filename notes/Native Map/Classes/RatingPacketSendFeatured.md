# Class: RatingPacketSendFeatured

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c82ad0` | `RatingPacketSendFeatured::RatingPacketSendFeatured` | ✅ | ctor | Initializes BaseRatingPacket with subcommand 0x16, installs RatingPacketSendFeatured vtable, initializes strings and zeroes adjacent fields. |
| `00c82b60` | `RatingPacketSendFeatured::Dtor ?` | 🟡 | dtor | Destroys embedded six-string data, releases another IString member, then restores BaseRatingPacket and BasePacket vtables. |
| `00c831f0` | `RatingPacketSendFeatured::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor and conditionally frees the object when flag bit 0 is set. |
