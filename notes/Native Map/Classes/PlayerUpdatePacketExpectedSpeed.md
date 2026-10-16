# Class: PlayerUpdatePacketExpectedSpeed

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0280` | `PlayerUpdatePacketExpectedSpeed::~PlayerUpdatePacketExpectedSpeed` | ✅ | dtor | Destructor body: restores PlayerUpdatePacketExpectedSpeed::vftable, then BasePlayerUpdatePacket::vftable, then BasePacket::vftable. |
| `008a02e0` | `PlayerUpdatePacketExpectedSpeed::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: restores the ExpectedSpeed, BasePlayerUpdatePacket and BasePacket vtables inside an SEH frame and calls _free(this) when param_2 bit 0 is set. |
| `008b3c10` | `— ⚠reject` | ⚪ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x17) and installs PlayerUpdatePacketExpectedSpeed::vftable; no other field is touched. |
