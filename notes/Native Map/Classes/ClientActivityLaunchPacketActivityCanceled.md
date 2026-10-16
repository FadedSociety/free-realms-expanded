# Class: ClientActivityLaunchPacketActivityCanceled

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009c0d00` | `ClientActivityLaunchPacketActivityCanceledCtor` | ✅ | ctor | Initializes the family packet with sub-opcode 6 and supplied fields, installs the canceled-packet vtable, and zeroes its trailing field. |
| `009c1260` | `ClientActivityLaunchPacketActivityCanceledScalarDeletingDtor` | ✅ | virtual | Rewrites the object vtable to BasePacket::vftable and frees the allocation when delete flag bit 0 is set. |
