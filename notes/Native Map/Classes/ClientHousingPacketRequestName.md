# Class: ClientHousingPacketRequestName

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad540` | `ClientHousingPacketRequestName::ctor` | ✅ | ctor | Constructor: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x11 (17, base opcode 127), stamps ClientHousingPacketRequestName::vftable at +0 and constructs one SoeUtil::IString<char> member a… |
| `00aad5b0` | `ClientHousingPacketRequestName::~ClientHousingPacketRequestName` | ✅ | dtor | Destructor: resets the vtable to ClientHousingPacketRequestName::vftable, releases the SoeUtil::IString<char> member at +0xc (refcount decrement, release through vtable slot +8) and finally resets the object's vtable to… |
| `00aad640` | `ClientHousingPacketRequestName::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0]): calls the class destructor FUN_00aad5b0 and frees the object through _free when the low bit of param_2 is set. |
| `00abcf40` | `ClientHousingPacketRequestName_SerializeBody` | 🟢 | function | Calls FUN_00aba910, serializes the string member at source+0xc with FUN_00911c30, then writes one byte from source+0x1c through FUN_007cc400. |
| `00ac1430` | `SendClientHousingPacketRequestName` | 🟢 | function | Transport wrapper for the house-name request: opens the scratch scope with FUN_008d4a40(0x10000000), runs the body writer FUN_00abcf40 over the caller's packet (one length-prefixed string block plus one byte), wraps the… |
