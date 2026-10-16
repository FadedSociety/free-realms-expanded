# Class: FreeInteractionNpc

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009d9a90` | `FreeInteractionNpc::FreeInteractionNpc` | ✅ | ctor | Constructor for FreeInteractionNpc: calls FUN_008a2820(0x14,0), the BaseCommandPacket constructor that installs BasePacket then BaseCommandPacket vtables and stores sub-opcode and an extra word, then installs FreeIntera… |
| `009db7f0` | `FreeInteractionNpc::ScalarDeletingDtor` | 🟢 | dtor | Scalar deleting destructor: rewrites the vtables to BaseCommandPacket::vftable then BasePacket::vftable and frees the object when (param_2 and 1) is set. Slot 0 of FreeInteractionNpc::vftable (dword at 0x018104dc is thi… |
