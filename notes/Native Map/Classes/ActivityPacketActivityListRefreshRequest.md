# Class: ActivityPacketActivityListRefreshRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a2330` | `ActivityPacketActivityListRefreshRequest::ActivityPacketActivityListRefreshRequ…` | ✅ | ctor | Constructor: calls FUN_008a21f0(4) so the packet is opcode 167 with activity sub-opcode 4, then installs ActivityPacketActivityListRefreshRequest::vftable. |
| `008a2390` | `ActivityPacketActivityListRefreshRequest::~ActivityPacketActivityListRefreshReq…` | 🟢 | dtor | Destructor body: restores BaseActivityPacket, BaseActivityServicePacket and BasePacket vtables; called by the request senders 009392c0 and 009471a0 plus four EH funclets. |
| `008b5f70` | `ActivityPacketActivityListRefreshRequest::ScalarDeletingDestructor` | 🟢 | dtor | Stores BaseActivityPacket, BaseActivityServicePacket, then BasePacket vtable pointers at *this; frees this only when deleting-flag bit 0 is set. |
