# Class: ProgressiveQuestClientDataPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdc970` | `ProgressiveQuestClientDataPacket::Ctor` | 🟢 | ctor | Constructs the packet with FUN_00bdc8a0(1) (family 0xcf, sub-opcode 1), installs ProgressiveQuestClientDataPacket::vftable and stores the argument (the ClientProgressiveQuestData record pointer) at +0xc. |
| `00bdc9e0` | `ProgressiveQuestClientDataPacket::~ProgressiveQuestClientDataPacket` | 🟢 | dtor | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by the unwind funclet 015efacb of the handler that builds the packet. |
| `00bdcef0` | `ProgressiveQuestClientDataPacket::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of ProgressiveQuestClientDataPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when the low bit of the flag argumen… |
