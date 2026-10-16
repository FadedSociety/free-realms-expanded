# Class: HousingPacketFixturePlacementDenied

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aad660` | `HousingPacketFixturePlacementDenied::ctor` | ✅ | ctor | Constructor of the placement-denied packet: runs the BaseHousingPacket base constructor FUN_00aa4990 with inner opcode 0x35 (53, base opcode 127), stamps HousingPacketFixturePlacementDenied::vftable at +0 and constructs… |
| `00aad6d0` | `HousingPacketFixturePlacementDenied::~HousingPacketFixturePlacementDenied` | 🟢 | dtor | Destructor: releases the SoeUtil::IString<char> member at +0xc (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseHousingPacket::vftable and BasePacket::vftable. |
| `00ab0920` | `HousingPacketFixturePlacementDeniedScalarDeletingDtor` | ✅ | dtor | Calls the placement-denied packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
