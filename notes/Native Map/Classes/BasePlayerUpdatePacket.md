# Class: BasePlayerUpdatePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0089fb50` | `BasePlayerUpdatePacket::CtorWithZeroSubOpcode` | 🟢 | ctor | Family constructor with a fixed zero sub-opcode: installs BasePacket::vftable, writes 0x23 (35) into param_1[1], installs BasePlayerUpdatePacket::vftable and clears param_1[2]. |
| `0089fbb0` | `BasePlayerUpdatePacket::BasePlayerUpdatePacket` | ✅ | ctor | Family constructor: installs BasePacket::vftable, stores 0x23 (opcode 35) into the packet id slot param_1[1], installs BasePlayerUpdatePacket::vftable, then stores the sub-opcode argument into param_1[2]. Writes nothing… |
| `0089fc20` | `BasePlayerUpdatePacket::~BasePlayerUpdatePacket` | ✅ | dtor | Destructor body: restores this->vtable to BasePlayerUpdatePacket::vftable and then to BasePacket::vftable (base destructor inlined). Touches no data fields and does not free. |
| `0089fc70` | `BasePlayerUpdatePacket::ScalarDeletingDestructor` | 🟢 | dtor | MSVC scalar deleting destructor: restores BasePlayerUpdatePacket then BasePacket vtables inside an SEH frame and, when the flag byte param_2 has bit 0 set, calls _free(this). |
| `008eb030` | `BasePlayerUpdatePacketDtorWithAnimationList` | ✅ | dtor | Destroys embedded animation list, then restores BasePlayerUpdatePacket and BasePacket vtables. |
