# Class: RatingPacketSearchReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c86710` | `RatingPacketSearchReplyCtor` | ✅ | ctor | Constructs BaseRatingPacket with subtype 0x0d, installs the search-reply vtable, and initializes its owned string fields. |
| `00c868e0` | `RatingPacketSearchReplyScalarDeletingDtor` | ✅ | virtual | Runs the search-reply destructor, frees the object when flag bit 0 is set, and otherwise returns it. |
