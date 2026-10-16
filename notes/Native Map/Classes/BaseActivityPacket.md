# Class: BaseActivityPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a21f0` | `BaseActivityPacket::BaseActivityPacket` | ✅ | ctor | Activity base constructor taking the activity sub-opcode: calls FUN_008a20c0(1) so the packet header carries opcode 167 with service sub-opcode 1, installs BaseActivityPacket::vftable and stores the argument in param_1[… |
| `008a2260` | `BaseActivityPacket::~BaseActivityPacket` | ✅ | dtor | Destructor body: restores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable then BasePacket::vftable; 13 EH funclets call it. |
| `008a22c0` | `BaseActivityPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the activity, activity-service and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `00bdeda0` | `BaseActivityPacket_Ctor` | 🟢 | ctor | Constructs the activity packet with FUN_008a20c0(1) (service 167, service-type 1) and installs BaseActivityPacket::vftable, storing 0 in the field at +0xc. |
| `00bdf000` | `BaseActivityPacket::~BaseActivityPacket` | 🟢 | dtor | Destructor body: stores BaseActivityPacket::vftable, BaseActivityServicePacket::vftable and finally BasePacket::vftable at object+0, with no member cleanup. |
