# Class: SoeUtil::List<MemberPanelDataSendPacket::MemberPanelData,-1>

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a3e8e0` | `List_MemberPanelData_AllocNode` | ✅ | virtual | Vtable slot-2 node allocator of the member-panel-data list: calls FUN_006fb9c0 and FUN_006fb860 and allocates a 0x3c-byte node through the manager's virtual slot 1, the global manager DAT_01bdf280 or the CRT fallback FU… |
| `00a3e920` | `List_MemberPanelData_FreeNode` | ✅ | virtual | One-argument node deallocator: calls FUN_00401c00(ptr,4) and returns. |
| `00a3f2c0` | `ListRecord_AppendNode_8_24` | 🟢 | method | Appends one node to the member-panel list: allocates it through the container's virtual slot +8, constructs the row through 00a3eec0, then links it at the tail using the +0x34 (next) and +0x38 (previous) fields with hea… |
| `00a3f760` | `ListRemoveNodeAndRunSlot3` | 🟢 | method | Rewires adjacent nodes through param_2 fields +0x34/+0x38, updates the list head or tail at param_1+4/+8 when a link is null, decrements count at +0xc, passes param_2 in ECX to FUN_00a3ebd0 to release its two IString fi… |
| `00a400e0` | `ReadCountedEntryListB` | 🟢 | method | Reads a count-prefixed list from the raw reader param_1 into the list at param_2: drains it through FUN_00a3f760, reads one dword as the entry count (checking the reader error flag at param_1+0x10 after each step), then… |
| `00a40350` | `SoeUtil::List::~List` | ✅ | dtor | Destructor of the MemberPanelData list embedded at +0xc of MemberPanelDataSendPacket: installs SoeUtil::List<MemberPanelDataSendPacket::MemberPanelData,-1>::vftable and removes/destroys head nodes through FUN_00a3f760 u… |
| `00a403c0` | `SoeUtil::List::ScalarDeletingDtor → SoeUtil::List<MemberPanelDataSendPacket::Me…` | ✅ | dtor | Scalar deleting destructor (vtable slot 0) of the member-panel list: installs the instantiation vtable, drains the list with FUN_00a3f760, and frees the object with _free when bit 0 of the delete flag is set. |
