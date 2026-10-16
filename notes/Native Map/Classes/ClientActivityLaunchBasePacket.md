# Class: ClientActivityLaunchBasePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c0bc0` | `ClientActivityLaunchBasePacketCtor` | ✅ | ctor | Installs BasePacket and ClientActivityLaunchBasePacket vtables, stores opcode 0xaf, and zeroes three packet fields. |
| `009c0c30` | `ClientActivityLaunchBasePacketInit` | ✅ | ctor | Stores opcode 0xaf, writes supplied fields in packet slots +3 and +4, and stores the activity-launch base vtable. |
| `009c0cb0` | `ClientActivityLaunchBasePacketDtor` | 🟢 | dtor | Rewrites the object vtable pointer to BasePacket::vftable and returns without freeing the object. |
| `009c1200` | `ClientActivityLaunchBasePacketScalarDeletingDtor` | ✅ | virtual | Rewrites the object vtable to BasePacket::vftable and frees the allocation when delete flag bit 0 is set. |
