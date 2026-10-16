# Class: MemberPanelDataSendPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a40590` | `MemberPanelDataSendPacket::MemberPanelDataSendPacket` | ✅ | ctor | Constructor: runs the announcement base constructor FUN_00a3df90(4) (installing BasePacket/AnnouncementBasePacket vtables, base opcode 0xc1 at +4 and sub-opcode 4 at +8), installs MemberPanelDataSendPacket::vftable and … |
| `00a40600` | `MemberPanelDataSendPacket::~MemberPanelDataSendPacket` | ✅ | dtor | Installs MemberPanelDataSendPacket::vftable, removes list-head nodes through FUN_00a3f760 until the +0x10 head is empty, reinstalls the embedded list vtable at +0x0c, then reaches a second loop whose saved head is alrea… |
| `00a406a0` | `MemberPanelDataSendPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of MemberPanelDataSendPacket: calls the destructor body 00a40600 and frees the packet with _free when bit 0 of the delete flag is set. |
| `00a408f0` | `ParseBufferIntoListB` | 🟢 | method | Builds a bounded reader over (param_2,param_3), parses the count-prefixed member-panel list into param_1+0xc through FUN_00a400e0, and returns 0 for null input or reader error. After a successful parse, nonzero param_4 … |
