# Class: HousingPacketInstanceList

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00abd380` | `HousingPacketInstanceList::HousingPacketInstanceList` | ✅ | ctor | Calls FUN_00aa4990 with 0x27, installs HousingPacketInstanceList and embedded List<PlayerHousingInstanceInfo,-1> vtables, and zeroes list head, tail and count at this+0x1c,+0x20,+0x24. |
| `00abd3f0` | `HousingPacketInstanceList::~HousingPacketInstanceList` | ✅ | dtor | Resets the embedded list vtable at this+0x18, removes each node through FUN_00ab6dd0, then rewrites this to BaseHousingPacket and BasePacket vtables. |
| `00abef00` | `HousingPacketInstanceList::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor of the instance-list packet: calls the destructor body 00abd3f0 in place and frees the object with _free when the low bit of the flag is set. Bytes decoded as 56 8b f1 e8 .. f6 44 24 08 01 74 … |
| `00abf210` | `— ⚠reject` | ⚪ | function | Body parser for the inbound instance list: builds the bounded reader over [param_2, param_2+param_3) with FUN_00ab2bc0, reads the record list at destination+0x18 through FUN_00aba800 (a count followed by that many eleme… |
