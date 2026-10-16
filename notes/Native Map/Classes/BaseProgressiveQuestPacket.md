# Class: BaseProgressiveQuestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdc8a0` | `BaseProgressiveQuestPacket::BaseProgressiveQuestPacket` | ✅ | ctor | Family constructor of the progressive-quest packet: stores BasePacket::vftable, writes the family opcode 0xcf (207) into the packet opcode field at +4, installs BaseProgressiveQuestPacket::vftable and stores the caller'… |
| `00bdc920` | `BaseProgressiveQuestPacket::~BaseProgressiveQuestPacket` | ✅ | dtor | Destructor body shared by the progressive-quest packet classes: stores BasePacket::vftable at object+0 and performs no member cleanup; referenced by 22 EH unwind funclets across the family. |
| `00bdce90` | `BaseProgressiveQuestPacket::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of BaseProgressiveQuestPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when the low bit of the flag argument is s… |
