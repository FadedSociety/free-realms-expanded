# Class: RatingPacketRequestFeatured

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aadab0` | `RatingPacketRequestFeatured::ctor` | ✅ | ctor | Constructor: runs the BaseRatingPacket base constructor FUN_00aa5ca0 with inner opcode 0x14 (20, base opcode 0xae = 174), stamps RatingPacketRequestFeatured::vftable at +0 and constructs one SoeUtil::IString<char> membe… |
| `00aadb20` | `RatingPacketRequestFeatured::~RatingPacketRequestFeatured` | 🟢 | dtor | Destructor: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseRatingPacket::vftable and BasePacket::vftable. |
| `00ab0980` | `RatingPacketRequestFeaturedScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac1770` | `SendRatingPacketRequestFeatured` | 🟢 | function | Transport wrapper for the rating featured-request packet: opens the scratch scope with FUN_008d4a40(0x10000000), emits the rating family header with FUN_00aba870, appends the string member at +0xc through FUN_00911c30 a… |
