# Class: RatingPacketVoteReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c81e30` | `RatingPacketVoteReply::RatingPacketVoteReply` | ✅ | ctor | Calls BaseRatingPacket initializer with subcommand 0x12, installs RatingPacketVoteReply::vftable, and returns the object pointer. |
| `00c81e90` | `RatingPacketVoteReply::ScalarDeletingDtor` | ✅ | dtor | Restores the BaseRatingPacket and BasePacket vtables, conditionally frees the object when flag bit 0 is set, and otherwise returns it. |
