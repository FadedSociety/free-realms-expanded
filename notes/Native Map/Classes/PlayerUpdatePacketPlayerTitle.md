# Class: PlayerUpdatePacketPlayerTitle

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a12c0` | `PlayerUpdatePacketPlayerTitle::~PlayerUpdatePacketPlayerTitle` | ✅ | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x28 and also runs from EH funclet 015914f3. |
| `008b50a0` | `PlayerUpdatePacketPlayerTitle::PlayerUpdatePacketPlayerTitle ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x28), installs PlayerUpdatePacketPlayerTitle::vftable and zeroes +0x18, +0x1c, +0x20 and +0x24. |
| `008b5110` | `PlayerUpdatePacketPlayerTitle::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
