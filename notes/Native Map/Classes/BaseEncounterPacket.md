# Class: BaseEncounterPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2f60` | `BaseEncounterPacket::BaseEncounterPacket` | ✅ | ctor | Encounter family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, stores 0x29 (41) in the packet id slot param_1[1], installs BaseEncounterPacket::vftable and zeroes param_1[2], param_1[3], param_… |
| `008a2fd0` | `BaseEncounterPacket::BaseEncounterPacketSub` | ✅ | ctor | Encounter family constructor taking the sub-opcode: installs BasePacket::vftable, stores 0x29 (41) in slot param_1[1], installs BaseEncounterPacket::vftable, stores the argument in param_1[2] and zeroes param_1[3], para… |
| `008a3040` | `BaseEncounterPacket::~BaseEncounterPacket` | ✅ | dtor | Destructor body: restores BaseEncounterPacket::vftable then BasePacket::vftable; reached from a block of EH unwind funclets around 0157d2xx, which is why it acts as the shared cleanup body for encounter packet locals. |
| `008a3090` | `BaseEncounterPacket::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the encounter and BasePacket vtables in an SEH frame, then calls _free(this) when param_2 bit 0 is set. |
| `008a32b0` | `BaseEncounterPacket::~BaseEncounterPacket` | 🟢 | dtor | Restores BaseEncounterPacket::vftable at +0 and BasePacket::vftable at +0, then returns. |
| `00a22360` | `BaseEncounterPacket::Dtor` | 🟢 | dtor | Destructor body that rewrites the object vtable through BaseEncounterPacket::vftable and then BasePacket::vftable; no members are released and no memory is freed. |
| `00a862a0` | `BaseEncounterPacketDtorBody` | ✅ | dtor | Destructor body of BaseEncounterPacket: chains the two vtable stores that MSVC emits for a destructor of a class deriving from BasePacket (own vtable first, then BasePacket's) and returns; called by the encounter packet… |
| `00aa2f20` | `BaseEncounterPacket::~BaseEncounterPacket` | ✅ | dtor | Calls FUN_009b40e0 with ECX=this+0x2f0 and no stack arguments, then calls FUN_00a284d0 with ECX=this+0x18 and no stack arguments, and finally stores BaseEncounterPacket::vftable then BasePacket::vftable at the object st… |
