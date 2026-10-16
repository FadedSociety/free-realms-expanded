# Class: RatingPacketCandidateInfoReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c856c0` | `RatingPacketCandidateInfoReplyCtor` | ✅ | ctor | Constructs BaseRatingPacket with subtype 0x11, installs the reply vtable, and initializes its candidate-info list pointers and count to zero. |
| `00c85730` | `RatingPacketCandidateInfoReplyDtor` | ✅ | dtor | Destroys candidate-info list nodes, then restores BaseRatingPacket and BasePacket vtables. |
| `00c86830` | `RatingPacketCandidateInfoReplyScalarDeletingDtor` | ✅ | virtual | Runs the candidate-info reply destructor, frees the object when flag bit 0 is set, and otherwise returns it. |
