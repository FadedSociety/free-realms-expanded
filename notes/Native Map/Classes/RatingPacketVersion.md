# Class: RatingPacketVersion

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c82ca0` | `RatingPacketVersion::RatingPacketVersion` | ✅ | ctor | Initializes BaseRatingPacket with subcommand 0x17, installs RatingPacketVersion vtable, and initializes one embedded IString and scalar fields. |
| `00c82d10` | `RatingPacketVersion::Dtor ?` | 🟡 | dtor | Releases the embedded IString value, then restores BaseRatingPacket and BasePacket vtables. |
| `00c83210` | `RatingPacketVersion::ScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor and conditionally frees the object when flag bit 0 is set. |
