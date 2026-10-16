# Class: RatingPacketRemoveCandidate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aade40` | `RatingPacketRemoveCandidate::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 7 (base opcode 0xae = 174), stamps RatingPacketRemoveCandidate::vftable at +0 and constructs two SoeUtil::IString<char> members at +… |
| `00aadec0` | `RatingPacketRemoveCandidate::~RatingPacketRemoveCandidate` | 🟢 | dtor | Destructor: releases the two SoeUtil::IString<char> members at +0xc and +0x1c (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseRatingPacket::vftable and BasePacket… |
| `00ab09c0` | `RatingPacketRemoveCandidateScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1650` | `SendRatingPacketRemoveCandidate` | 🟢 | function | Transport wrapper for the rating remove-candidate packet: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00aba870 and appends the two string members at +0xc and +0x1c thro… |
