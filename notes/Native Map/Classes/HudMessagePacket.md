# Class: HudMessagePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a1900` | `HudMessagePacket::~HudMessagePacket` | 🟢 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x40 and also runs from EH funclet 015917eb. |
| `008b5aa0` | `HudMessagePacket::HudMessagePacket ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x40), installs HudMessagePacket::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and again into +0x18/+0x1c, and zeroes +0x20, +0x24, +0x28, +0… |
| `008b5b30` | `HudMessagePacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
