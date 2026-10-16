# Class: ActivityProfileListPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00be5490` | `ActivityProfileListPacket::Ctor` | ✅ | ctor | Constructor body: calls FUN_00bdeb20 with sub-opcode 1 (the ActivityManagerBase sub-opcode constructor), stores the ActivityProfileListPacket vtable at *this, and constructs the embedded 20-bucket ActivityForProfileType… |
| `00be54f0` | `ActivityProfileListPacket::DtorBody` | ✅ | dtor | Destructor body with an unwind frame: stores the ActivityProfileListPacket vtable, empties the embedded ActivityForProfileType hash list through 00be2220 (after storing its HashListMap vtable at this+0xc), then stores t… |
| `00be5570` | `ActivityProfileListPacket::ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor (ret 4): calls 00be54f0 (the packet destructor body) on this, calls _free when bit 0 of the flag argument is set, and returns this. |
