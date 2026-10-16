# Class: HousingPacketZoneData

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aacdc0` | `ZoneDataPayloadInit` | 🟢 | method | Initialises the zone-data payload: takes four dwords from globals at +0/+4/+8/+0xc, zeroes dwords at +0x10/+0x34/+0x38 and bytes at +0x48/+0x49/+0x4a, constructs five SoeUtil::IString<char> members at +0x14, +0x24, +0x4… |
| `00aad3a0` | `ZoneDataPayloadRelease` | 🟢 | method | Releases the five SoeUtil::IString<char> members of the zone-data payload at +0x14, +0x24, +0x4c, +0x5c and +0x6c: for each it resets the member vtable, and when the capacity word is positive it decrements the reference… |
| `00aad4e0` | `HousingPacketZoneData::~HousingPacketZoneData` | 🟢 | dtor | Destructor of the zone-data packet: releases the five embedded SoeUtil::IString<char> members through FUN_00aad3a0 and then resets the object's vtable to BaseHousingPacket::vftable and BasePacket::vftable. |
| `00ab07a0` | `HousingPacketZoneDataCtor` | ✅ | ctor | Constructs the base housing packet with subtype 0x2d, installs the HousingPacketZoneData vtable, initializes its payload defaults, and sets two byte fields and a word to their initial values. |
| `00ab0820` | `HousingPacketZoneDataScalarDeletingDtor` | ✅ | dtor | Calls the zone-data packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
