# Class: PlayerUpdatePacketSetLookAt

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a0b40` | `PlayerUpdatePacketSetLookAt::~PlayerUpdatePacketSetLookAt` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; called from dispatcher case 0x11 and EH funclet 015915cd. |
| `008b4440` | `PlayerUpdatePacketSetLookAt::PlayerUpdatePacketSetLookAt` | ✅ | ctor | Calls the opcode-35 family constructor 0089fbb0(0x11) and installs PlayerUpdatePacketSetLookAt::vftable; no field is initialised. |
| `008b44a0` | `PlayerUpdatePacketSetLookAt::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
