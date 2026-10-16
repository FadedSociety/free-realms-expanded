# Class: StopInspectPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3bd0` | `StopInspectPacket::StopInspectPacket` | ✅ | ctor | Constructor: calls FUN_008a3b00(2) so the packet is inspect opcode 149 with sub-opcode 2, then installs StopInspectPacket::vftable; its only caller is 009347d0. |
| `008a3c30` | `StopInspectPacket::~StopInspectPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable only; its only caller is EH unwind funclet 015918c0. |
| `008b7730` | `StopInspectPacket::ScalarDeletingDestructor` | 🟢 | dtor | Stores BasePacket::vftable into *this and calls operator delete on this when bit 0 of the flag argument is set; returns this. |
