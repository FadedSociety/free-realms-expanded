# Class: BeginJobCustimzationPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4f50` | `BeginJobCustimzationPacket::BeginJobCustimzationPacket` | ✅ | ctor | Constructor: calls FUN_008a4e90(1) so the packet is opcode 155 with sub-opcode 1, then installs BeginJobCustimzationPacket::vftable; SEH frame restores ExceptionList. |
| `008a4fb0` | `BeginJobCustimzationPacket::~BeginJobCustimzationPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable and returns; the 009471a0 router destroys the begin-customization packet with it at the end of the case, and funclets 01593c3d also call it. |
| `008b8040` | `BeginJobCustimzationPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
