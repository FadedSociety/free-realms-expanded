# Class: BaseReportPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a5650` | `BaseReportPacket::BaseReportPacket` | ✅ | ctor | Report family constructor: installs BasePacket::vftable, stores opcode 68 (0x44) at +4, installs BaseReportPacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a56c0` | `BaseReportPacket::~BaseReportPacket` | ✅ | dtor | Destructor body: restores BaseReportPacket::vftable then BasePacket::vftable; four EH funclets (01584c10, 01586130, 015c3c90, 015c3cd0) call it. |
| `008a5710` | `BaseReportPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor: in an SEH frame restores BaseReportPacket::vftable then BasePacket::vftable and calls _free(this) when param_2 bit 0 is set. |
| `008d1e10` | `BaseReportPacket::~BaseReportPacket` | ✅ | dtor | Destructor body of the report-packet base: first runs the CRT vector-destructor iterator over the ten inline 0x30-byte records starting at +0x20 (element destructor FUN_008d1d60), then releases the packet's own SoeUtil:… |
