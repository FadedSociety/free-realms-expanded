# Class: LeaveLobbyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00bafe60` | `LeaveLobbyPacket_Ctor` | ✅ | ctor | Constructor: calls 008a3ce0 with argument 9 on ECX (lobby packet base constructor), stores the LeaveLobbyPacket vtable (01824f3c) at +0 and the parameter at +0xc. |
| `00bb00e0` | `LeaveLobbyPacket_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (vtable slot [0] of LeaveLobbyPacket): stores the BasePacket vtable (018011ec) and calls _free on the object when the delete flag bit 0 is set. |
