# Class: ReferenceDataPacketItemClassDefinitions

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4680` | `ReferenceDataPacketItemClassDefinitions::ReferenceDataPacketItemClassDefinitions` | ✅ | ctor | Item-class-definitions packet constructor taking a payload pointer: calls FUN_008a4550(1) so the packet is opcode 44 with sub-opcode 1, installs ReferenceDataPacketItemClassDefinitions::vftable and stores the payload po… |
| `008a46f0` | `ReferenceDataPacketItemClassDefinitions::~ReferenceDataPacketItemClassDefinitio…` | 🟢 | dtor | Destructor body: writes ReferenceDataPacket::vftable then BasePacket::vftable into the object vtable slot and returns; the 00928d00 dispatcher and EH funclet 01590030 call it. |
| `008b7cb0` | `ReferenceDataPacketItemClassDefinitions::ScalarDeletingDtor` | ✅ | dtor | Restores ReferenceDataPacket::vftable then BasePacket::vftable, and frees the object when the deleting flag bit is set. |
