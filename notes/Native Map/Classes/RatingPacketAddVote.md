# Class: RatingPacketAddVote

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aadf80` | `RatingPacketAddVote::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 8 (base opcode 0xae = 174), stamps RatingPacketAddVote::vftable at +0 and constructs four SoeUtil::IString<char> members at +0xc, +0… |
| `00aae020` | `RatingPacketAddVote::~RatingPacketAddVote` | 🟢 | dtor | Destructor: releases the four SoeUtil::IString<char> members at +0xc, +0x1c, +0x2c and +0x3c (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseRatingPacket::vftable… |
| `00ab09e0` | `RatingPacketAddVoteScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1890` | `SendRatingPacketAddVote` | 🟢 | function | Transport wrapper for the rating add-vote packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abd070 over the caller's packet (four string blocks plus a trailing dword), wraps the p… |
