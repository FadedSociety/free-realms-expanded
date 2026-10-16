# Class: ActivityPacketListOfActivities

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be5590` | `ActivityPacketListOfActivities::Ctor` | ✅ | ctor | Constructor body: calls FUN_008a21f0 with the constant 1 (the activity base constructor with sub-opcode), stores the ActivityPacketListOfActivities vtable at *this, stores the List<ClientActivityDefinition,-1> vtable at… |
| `00be6190` | `ActivityPacketListOfActivities_ScalarDtor` | 🟢 | dtor | Calls 00be5600 (list destructor body) on this, then calls _free(this) when bit 0 of the flag argument is set (ret 4); returns this. |
