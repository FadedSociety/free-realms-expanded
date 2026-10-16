# Class: BaseClaimPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff290` | `BaseClaimPacket::BaseClaimPacket ⚠downgrade` | 🟢 | ctor | Family constructor: stores BasePacket::vftable, writes the base opcode 0x6c (108) into the packet id slot param_1[1], stores BaseClaimPacket::vftable and then stores the caller-supplied sub-opcode in param_1[2]. |
| `009ff350` | `BaseClaimPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor inside an SEH frame: stores BaseClaimPacket::vftable, then BasePacket::vftable, then _free(this) when bit 0 of the flag is set. |
