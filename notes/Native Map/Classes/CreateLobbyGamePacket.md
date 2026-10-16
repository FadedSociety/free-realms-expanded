# Class: CreateLobbyGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bb0b80` | `CreateLobbyGamePacket_Ctor` | ✅ | ctor | Constructor: calls 008a3ce0 with argument 5 on ECX (lobby packet base constructor), stores the CreateLobbyGamePacket vtable (01824fc8), copies the source record at param_2 through 00bb0a30, and stores the three paramete… |
| `00bb0c10` | `— ⚠reject` | ⚪ | dtor | Scalar deleting destructor (CreateLobbyGamePacket vtable slot [0]): calls 00bb06d0 (BasePacket destructor body) on ECX and calls _free when the delete flag bit 0 is set. |
