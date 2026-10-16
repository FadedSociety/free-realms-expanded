# Class: BaseSoccerPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b73040` | `— ⚠reject` | ⚪ | ctor | Initialises a soccer packet object: stores BasePacket::vftable, writes 0x4c (family 76) to +4, stores BaseSoccerPacket::vftable and zeroes +8 (sub id 0). |
| `00b730a0` | `— ⚠reject` | ⚪ | ctor | Base constructor of every family-76 soccer packet: stores BasePacket::vftable, writes 0x4c (family 76) to +4, stores BaseSoccerPacket::vftable and writes the caller's sub id to +8. |
| `00b73110` | `BaseSoccerPacket::~BaseSoccerPacket` | ✅ | dtor | Base-object destructor body of the family-76 packet hierarchy: stores BaseSoccerPacket::vftable, marks the object dead for EH bookkeeping (local_4 = 0xffffffff) and then stores BasePacket::vftable. |
| `00b73160` | `BaseSoccerPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the object's vtable to BaseSoccerPacket::vftable then BasePacket::vftable and, when the flag byte's low bit is set, calls _free on the object. |
| `00b73950` | `BaseSoccerPacket::~BaseSoccerPacket` | ✅ | dtor | Stores BaseSoccerPacket::vftable and then BasePacket::vftable into the object. |
