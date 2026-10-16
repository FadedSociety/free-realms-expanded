# Class: BaseHousingPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aa4930` | `BaseHousingPacket::BaseHousingPacket` | ✅ | ctor | Stores BasePacket::vftable, sets the family opcode field to 0x7f (127), then stores BaseHousingPacket::vftable and zeroes the sub-opcode field. |
| `00aa4990` | `BaseHousingPacket::BaseHousingPacketSub` | ✅ | ctor | Stores BasePacket::vftable, sets the family opcode field to 0x7f (127), stores BaseHousingPacket::vftable, then writes the sub-opcode argument into object+0x8. |
| `00aa4a00` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4a50` | `BaseHousingPacket::ScalarDeletingDtor` | ✅ | dtor | Stores BaseHousingPacket::vftable and BasePacket::vftable, then calls _free on the object when bit 0 of param_2 is set. |
| `00aa4b20` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4bd0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4c20` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4c70` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4d20` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4dd0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4e20` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4ed0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa4f80` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5030` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa50f0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa51a0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa51f0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5240` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa52f0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa53a0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5450` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa54a0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5550` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa55a0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5660` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5710` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5760` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5810` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5930` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa59e0` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aa5a90` | `BaseHousingPacket::~BaseHousingPacket` | ✅ | dtor | Stores BaseHousingPacket::vftable and then BasePacket::vftable into the object. |
| `00aad1a0` | `BaseHousingPacket::~BaseHousingPacket` | 🟢 | dtor | Base destructor: calls the base-packet destructor FUN_00777340 and resets the object's vtable to BaseHousingPacket::vftable and then BasePacket::vftable. |
| `00ab0580` | `BaseHousingPacketDtor ? ⚠rename` | 🟡 | dtor | Calls the notification member cleanup helper, then replaces the object's vtable with BaseHousingPacket and BasePacket base vtables. |
| `00ab05e0` | `BaseHousingPacketDtorVariant ? ⚠rename` | 🟡 | dtor | Calls the owned-member cleanup routine, then replaces the object's vtable with BaseHousingPacket and BasePacket base vtables. |
| `00aba910` | `BaseHousingPacket::WriteHeader` | ✅ | method | Thiscall on a housing packet object: writes its base opcode at +4 as a u16, then its inner opcode at +8 as a u16 into the writer argument, clipping each chunk to min(remaining, 2). The Ghidra dword indexes 0x806 and 0x8… |
