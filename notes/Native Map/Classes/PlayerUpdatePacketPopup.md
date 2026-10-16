# Class: PlayerUpdatePacketPopup

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a19b0` | `PlayerUpdatePacketPopup::~PlayerUpdatePacketPopup ? ⚠downgrade` | 🟡 | dtor | Destructor body: restores BasePlayerUpdatePacket::vftable then BasePacket::vftable; it is the last call of dispatcher case 0x46 and also runs from EH funclet 015915b7. |
| `008b5ca0` | `PlayerUpdatePacketPopup::PlayerUpdatePacketPopup ⚠downgrade` | 🟢 | ctor | Calls the opcode-35 family constructor 0089fbb0(0x46), installs PlayerUpdatePacketPopup::vftable, stores the global pair DAT_01be4850/DAT_01be4854 into +0x10/+0x14 and zeroes +0x18. |
| `008b5d20` | `PlayerUpdatePacketPopup::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePlayerUpdatePacket::vftable then BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; no direct callers. |
