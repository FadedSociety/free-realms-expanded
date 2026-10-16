# Class: BaseActivityServicePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a20c0` | `BaseActivityServicePacket::BaseActivityServicePacket` | ✅ | ctor | Activity-service family constructor: installs BasePacket::vftable, stores 0xa7 (167) in the packet id slot param_1[1], installs BaseActivityServicePacket::vftable and stores the sub-opcode argument in param_1[2]. |
| `008a2130` | `BaseActivityServicePacket::~BaseActivityServicePacket` | ✅ | dtor | Destructor body: restores BaseActivityServicePacket::vftable then BasePacket::vftable; called from 18 EH unwind funclets, which is why it is the shared destructor body for the activity families. |
| `008a2180` | `BaseActivityServicePacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the activity-service and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `00bded40` | `BaseActivityServicePacket_Ctor` | ✅ | ctor | Family constructor of the activity-service packets: stores BasePacket::vftable, writes the family opcode 0xa7 (167) into the packet field at +4, installs BaseActivityServicePacket::vftable and zeroes the service field a… |
