# Class: RatingPacketDataReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c86670` | `RatingPacketDataReplyCtor` | ✅ | ctor | Constructs BaseRatingPacket with subtype 5, installs the data-reply vtable, initializes an owned string, and initializes rating data storage. |
| `00c866f0` | `RatingPacketDataReplyScalarDeletingDtor` | ✅ | virtual | Runs the packet destructor, frees the object when flag bit 0 is set, and otherwise returns the object pointer. |
