# Class: ActivityPacketJoinActivityRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdefa0` | `ActivityPacketJoinActivityRequest_Ctor` | 🟢 | ctor | Constructs the request with FUN_008a21f0(2) (the BaseActivityPacket constructor with activity sub-opcode 2) and installs ActivityPacketJoinActivityRequest::vftable. |
| `00be0120` | `ActivityPacketJoinActivityRequest::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: resets the vftable to BaseActivityPacket, then BaseActivityServicePacket, then BasePacket; frees this through _free (thunk) only when bit 0 of the flag argument is set; returns this. |
