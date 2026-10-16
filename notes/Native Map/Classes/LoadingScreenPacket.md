# Class: LoadingScreenPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a8b820` | `LoadingScreenPacket::Construct` | ✅ | ctor | Constructs a LoadingScreenPacket: calls the BaseUiPacket constructor 00a86640 with sub-opcode 9, stores LoadingScreenPacket::vftable at *this, initialises an empty IString<char> at +0xc and another at +0x1c, then assign… |
| `00a8b8c0` | `LoadingScreenPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of LoadingScreenPacket: calls 00a89940 (PacketIStringPairDtorBody), then calls _free(this) through thunk 00d82463 when bit 0 of param_2 is set; returns this. |
