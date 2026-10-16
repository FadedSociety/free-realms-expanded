# Class: HousingPacketFixtureUpdate

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad200` | `HousingPacketFixtureUpdate::ctor` | ✅ | ctor | Constructor of the fixture-update packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x28 (40, base opcode 127), stores the three constructor arguments at +0xc/+0x10/+0x14, zeroes +0x18 a… |
| `00aad290` | `HousingPacketFixtureUpdate::~HousingPacketFixtureUpdate` | 🟢 | dtor | Destructor of the fixture-update packet: clears the dword at +0x24, releases the SoeUtil::Array<StrongType<unsigned __int64,SoeUtilSignatureCharacterGuid,0>,0,1> member at +0x1c (vtable reset, buffer freed with 8-byte a… |
| `00ab0660` | `HousingPacketFixtureUpdateScalarDeletingDtor` | ✅ | dtor | Calls the fixture-update destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
