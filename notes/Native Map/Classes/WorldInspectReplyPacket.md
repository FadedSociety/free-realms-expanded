# Class: WorldInspectReplyPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a3c80` | `WorldInspectReplyPacket::~WorldInspectReplyPacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable only; 009471a0 destroys the reply object with it inside the branch that constructs WorldInspectReplyPacket (FUN_008b7790, which calls FUN_008a3b00(4)) and applies the reply. |
| `008b7790` | `WorldInspectReplyPacket::WorldInspectReplyPacket` | ✅ | ctor | Calls the inspect packet base constructor with 4, installs WorldInspectReplyPacket::vftable, and initializes field +0x18 to 0xffffffff. |
| `008b77f0` | `WorldInspectReplyPacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
