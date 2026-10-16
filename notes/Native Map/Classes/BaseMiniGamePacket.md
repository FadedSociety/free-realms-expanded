# Class: BaseMiniGamePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4420` | `BaseMiniGamePacket::BaseMiniGamePacket` | ✅ | ctor | Minigame packet-family constructor: installs BasePacket::vftable, stores the family opcode 39 (0x27) at +4, stores the sub-opcode argument at +8 and the next argument at +0xc, then installs BaseMiniGamePacket::vftable, … |
| `008a44a0` | `BaseMiniGamePacket::~BaseMiniGamePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable into the object vtable slot and returns; 73 EH unwind funclets call it, and the 009471a0 router also calls it right after constructing minigame packets. |
| `008b7c40` | `BaseMiniGamePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
