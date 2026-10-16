# Class: SoeUtil::List<AnnouncementDataSendPacket::AnnouncementPacketData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e870` | `List_AnnouncementPacketData_AllocNode` | ✅ | virtual | Vtable slot-2 node allocator of the announcement-packet-data list: calls FUN_006fb9c0 then FUN_006fb860, uses the memory manager's virtual slot 1 when present, otherwise the global manager DAT_01bdf280, otherwise the CR… |
| `00a3e8b0` | `List_AnnouncementPacketData_FreeNode` | ✅ | virtual | One-argument node deallocator: calls FUN_00401c00(ptr,4) and returns. |
| `00a3f230` | `ListRecord_AppendNode_18_28_44` | 🟢 | method | Appends one node to the list: allocates it through the container's virtual slot +8, constructs the row through 00a3ee70, then links it at the tail using the +0x54 (next) and +0x58 (previous) fields with head at containe… |
| `00a3f6d0` | `ListRemoveNodeAndRunSlot3` | 🟢 | method | Rewires adjacent nodes through param_2 fields +0x54/+0x58, updates the list head or tail at param_1+4/+8 when a link is null, decrements count at +0xc, passes param_2 in ECX to FUN_00a3eaf0 to release its three IString … |
| `00a40070` | `ReadCountedEntryListA` | 🟢 | method | Reads a count-prefixed list of records from the raw reader param_1 into the list at param_2: drains the list through FUN_00a3f6d0, reads one dword as the entry count (honouring the reader error flag at param_1+0x10), th… |
| `00a40320` | `SoeUtil::List::~List` | ✅ | dtor | Destructor of the AnnouncementPacketData list embedded at +0xc of AnnouncementDataSendPacket: installs SoeUtil::List<AnnouncementDataSendPacket::AnnouncementPacketData,-1>::vftable at +0 and then repeatedly removes and … |
| `00a40380` | `SoeUtil::List::ScalarDeletingDtor → SoeUtil::List<AnnouncementDataSendPacket::A…` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the announcement list: installs the instantiation vtable, drains the list with FUN_00a3f6d0, returns the object when the delete flag is clear and otherwise frees it with _fr… |
