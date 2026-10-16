# Class: ActivityJoinErrorPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bdec00` | `ActivityJoinErrorPacket_Ctor` | 🟢 | ctor | Constructs the opcode-147 sub-opcode-2 packet with FUN_00bdeb20(2) and installs ActivityJoinErrorPacket::vftable. |
| `00bdec60` | `ActivityJoinErrorPacket::~ActivityJoinErrorPacket` | 🟢 | dtor | Destructor body: stores BasePacket::vftable at object+0 and performs no member cleanup; called at the end of the join-error branch of 00be7360 and referenced by the unwind funclet 015f0f4b. |
| `00be0090` | `ActivityJoinErrorPacket::ScalarDeletingDtor` | ✅ | virtual | Virtual scalar deleting destructor (slot 0 of ActivityJoinErrorPacket::vftable): stores BasePacket::vftable at object+0 and frees the object through the 00d82463 _free thunk when the low bit of the flag argument is set. |
