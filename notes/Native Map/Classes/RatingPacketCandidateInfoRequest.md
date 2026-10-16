# Class: RatingPacketCandidateInfoRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aae280` | `RatingPacketCandidateInfoRequest::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0x10 (16, base opcode 0xae = 174), stamps RatingPacketCandidateInfoRequest::vftable at +0, constructs two SoeUtil::IString<char> mem… |
| `00aae320` | `RatingPacketCandidateInfoRequest::~RatingPacketCandidateInfoRequest` | 🟢 | dtor | Destructor: releases the two SoeUtil::IString<char> members at +0xc and +0x1c (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseRatingPacket::vftable and BasePacket… |
| `00ab0a20` | `RatingPacketCandidateInfoRequestScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00abfe70` | `— ⚠reject` | ⚪ | function | Transport wrapper for the rating candidate-info request: opens the shared transport scratch scope with FUN_008d4a40(0x10000000), runs the rating family header writer FUN_00aba870 over the caller's packet, appends the tw… |
