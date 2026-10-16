# Class: ClaimItemsRequestPacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff630` | `ClaimItemsRequestPacket::~ClaimItemsRequestPacket` | ✅ | dtor | Destructor body: rewrites the vtable to ClaimItemsRequestPacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from the senders 00a04fd0/00a052e0 and from three unwind funclets. |
| `009ff690` | `ClaimItemsRequestPacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: restores ClaimItemsRequestPacket::vftable, BaseClaimPacket::vftable, BasePacket::vftable and frees this when bit 0 of the flag is set. |
| `00a00420` | `ClaimItemsRequestPacket::ClaimItemsRequestPacket ⚠downgrade` | 🟢 | ctor | Constructor: runs the base constructor 009ff290 with sub-opcode 3, stores ClaimItemsRequestPacket::vftable, then stores the two caller arguments at param_1[3] and param_1[4]. |
