# Class: JobCustomizationBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `008a4e90` | `JobCustomizationBasePacket::JobCustomizationBasePacket` | ✅ | ctor | Job-customization family constructor: installs BasePacket::vftable, stores opcode 155 (0x9b) at +4, installs JobCustomizationBasePacket::vftable and stores the sub-opcode argument at +8; no calls. |
| `008a4f00` | `JobCustomizationBasePacket::~JobCustomizationBasePacket` | 🟢 | dtor | Destructor body: writes BasePacket::vftable and returns; seven EH funclets (0157e110 and neighbours) call it. |
| `008b7fe0` | `JobCustomizationBasePacket::ScalarDeletingDtor` | ✅ | dtor | Restores BasePacket::vftable and frees the object when the deleting flag bit is set. |
