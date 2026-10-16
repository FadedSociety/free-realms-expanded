# Class: BaseMiniGameBossPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c47db0` | `— ⚠reject` | ⚪ | method | Constructs a boss packet: calls the minigame packet base constructor FUN_008a4420 with the sub-opcode 0x3e (62), StateId -1, the caller's discriminator argument and another -1 (the base stores the family opcode 0x27 = 3… |
| `00c47e20` | `BaseMiniGameBossPacket_Destruct` | 🟢 | dtor | Destructor body of the boss packet base temporary: the only operation is writing BasePacket::vftable into the object at +0 (base subobject teardown); it is referenced from seven EH unwind funclets of the handler 00c48c9… |
| `00c480d0` | `BaseMiniGameBossPacket_ScalarDeletingDtor` | 🟢 | virtual | Virtual slot 0 deleting destructor: writes BasePacket::vftable into the object and frees it with _free when the low bit of the flag argument is set. |
| `00c48620` | `BaseMiniGameBossPacket_ReadPayload` | 🟢 | method | Reads a mini-game message header with the shared header reader 009b2340 and then three more 32-bit payload fields into this+0x18, +0x1c and +0x20. Each read is bounds-checked against the stream end (stream+0xc): on over… |
