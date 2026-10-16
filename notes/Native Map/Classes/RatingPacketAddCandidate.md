# Class: RatingPacketAddCandidate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aadbb0` | `RatingPacketAddCandidate::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 6 (base opcode 0xae = 174), stamps RatingPacketAddCandidate::vftable at +0 and constructs seven SoeUtil::IString<char> members at +0… |
| `00aadc90` | `RatingPacketAddCandidate::~RatingPacketAddCandidate` | 🟢 | dtor | Destructor: releases the seven SoeUtil::IString<char> members at +0xc, +0x1c, +0x2c, +0x3c, +0x4c, +0x5c and +0x6c (each: vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable … |
| `00ab09a0` | `RatingPacketAddCandidateScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1540` | `SendRatingPacketAddCandidate` | 🟢 | function | Transport wrapper for the rating add-candidate packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcfa0 over the caller's packet, wraps the produced buffer in a tunneled ClientGat… |
