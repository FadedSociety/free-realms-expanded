# Class: ReportPacketReportHouse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00aae4f0` | `ReportPacketReportHouse::ctor` | ✅ | ctor | Constructor: runs the BaseReportPacket base constructor FUN_008a5650 with inner opcode 3 (base opcode 0x44 = 68), stamps ReportPacketReportHouse::vftable at +0, stores the global dword pair at +0x18/+0x1c and constructs… |
| `00aae580` | `ReportPacketReportHouse::~ReportPacketReportHouse` | 🟢 | dtor | Destructor: releases the two SoeUtil::IString<char> members at +0x20 and +0x30 (vtable reset, refcount decrement, release through vtable slot +8) and resets the object's vtable to BaseReportPacket::vftable and BasePacke… |
| `00ab0a40` | `ReportPacketReportHouseScalarDeletingDtor` | ✅ | dtor | Calls the packet destructor, conditionally frees the object when flag bit zero is set, and otherwise returns the original pointer. |
| `00ac19a0` | `SendReportPacketReportHouse` | 🟢 | function | Transport wrapper for the house-report packet: opens the scratch scope with FUN_008d4a40(0x10000000), runs the family header writer FUN_0090b190, writes the 64-bit field at +0x10 through FUN_008fd1a0 and the 64-bit fiel… |
