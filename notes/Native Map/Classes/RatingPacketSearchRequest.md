# Class: RatingPacketSearchRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aae140` | `RatingPacketSearchRequest::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0xc (12, base opcode 0xae = 174), stamps RatingPacketSearchRequest::vftable at +0 and constructs two SoeUtil::IString<char> members … |
| `00aae1c0` | `RatingPacketSearchRequest::~RatingPacketSearchRequest` | 🟢 | dtor | Destructor: releases the two SoeUtil::IString<char> members at +0x18 and +0x28 (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseRatingPacket::vftable and BasePacke… |
| `00ab0a00` | `RatingPacketSearchRequestScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1ae0` | `SendRatingPacketSearchRequest` | 🟢 | function | Transport wrapper for the rate-limited rating search request: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00aba870, appends the 64-bit field at +0x10 through FUN_008fb8… |
