# Class: BaseInspectPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3b00` | `BaseInspectPacket::BaseInspectPacket` | ✅ | ctor | Inspect family constructor: installs BasePacket::vftable, stores 0x95 (149) in the packet id slot param_1[1], installs BaseInspectPacket::vftable and stores the sub-opcode argument in param_1[2]. |
| `008a3b70` | `BaseInspectPacket::~BaseInspectPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable only; it is the last call of the inspect case in 009471a0 and is reached from several EH funclets (0157d740 and neighbours), so it acts as the shared inspect-packet cleanup bo… |
| `008b76b0` | `BaseInspectPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
