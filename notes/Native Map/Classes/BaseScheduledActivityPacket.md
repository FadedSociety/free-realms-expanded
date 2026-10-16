# Class: BaseScheduledActivityPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdee00` | `BaseScheduledActivityPacket_Ctor` | 🟢 | ctor | Constructs the scheduled-activity packet with FUN_008a20c0(2) (service 167, service-type 2) and installs BaseScheduledActivityPacket::vftable, storing 0 in the field at +0xc. |
| `00bdee60` | `BaseScheduledActivityPacket_CtorWithPayload` | 🟢 | ctor | Variant of the scheduled-activity constructor that also stores its argument in the packet field at +0xc after installing BaseScheduledActivityPacket::vftable. |
| `00bdeed0` | `BaseScheduledActivityPacket::~BaseScheduledActivityPacket` | 🟢 | dtor | Destructor body of the scheduled-activity family: stores BaseScheduledActivityPacket::vftable, then BaseActivityServicePacket::vftable and finally BasePacket::vftable at object+0, with no member cleanup. |
| `00bdef30` | `BaseScheduledActivityPacket::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of BaseScheduledActivityPacket::vftable): stores the three vtable levels (BaseScheduledActivityPacket, BaseActivityServicePacket, BasePacket) at object+0 and frees the object t… |
