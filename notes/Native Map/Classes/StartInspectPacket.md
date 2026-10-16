# Class: StartInspectPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008d1860` | `StartInspectPacket::StartInspectPacket` | ✅ | ctor | Constructor of the start-inspect packet: calls the inspect family constructor FUN_008a3b00 with sub-opcode 1, installs StartInspectPacket::vftable, stores a zero byte at +0xc, loads the two dwords at +0x10/+0x14 from th… |
| `008d18f0` | `StartInspectPacket::~StartInspectPacket` | 🟢 | dtor | Destructor body of the start-inspect packet: rewrites the embedded SoeUtil::IString<char> member at +0x18 (param_1[6], buffer +0x1c, capacity +0x24) to its vtable, LOCK-decrements the shared refcount and calls the strin… |
| `008dd230` | `StartInspectPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor: runs the destructor body 008d18f0 and frees the object with _free (00d82463) when the low bit of the flag argument is set. |
