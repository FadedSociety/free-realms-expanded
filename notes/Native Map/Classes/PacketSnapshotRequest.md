# Class: PacketSnapshotRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `0091d710` | `PacketSnapshotRequest::InitFields` | 🟢 | method | Initializes three SoeUtil::IString<char> members of a large object at this+8, this+0x700e0 and this+0x700f0 (vtable plus empty-buffer pointer) and calls FUN_009188e0 once between the first and the remaining two. |
| `0091d7d0` | `PacketSnapshotRequest::CleanupFields` | 🟢 | dtor | Releases the three SoeUtil::IString<char> members of the large object (+0x700f0, +0x700e0 via refcounted buffer release and +8 last), with a call to FUN_00919330 between the +0x700e0 and +8 releases. |
| `0091d8c0` | `PacketSnapshotRequest::Ctor` | ✅ | ctor | Constructor: runs the BaseFotomatPacket family constructor FUN_008a23f0(4) (opcode 156 with sub-type 4), installs PacketSnapshotRequest::vftable and then initializes the large embedded snapshot buffer via FUN_0091d710. |
| `009229f0` | `PacketSnapshotRequest_ScalarDtor` | ✅ | dtor | Scalar deleting destructor: calls the body 0091d920 on this, then frees the object through _free when bit 0 of the second argument is set; returns this. |
