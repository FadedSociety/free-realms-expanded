# Class: GetAllAvailableClaimItemsResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff4f0` | `GetAllAvailableClaimItemsResponsePacket::GetAllAvailableClaimItemsResponsePacke…` | 🟢 | ctor | Constructor: runs the base constructor 009ff290 with sub-opcode 2, clears the byte at +0xc, stores GetAllAvailableClaimItemsResponsePacket::vftable and stores the caller's argument in param_1[4]. |
| `009ff560` | `GetAllAvailableClaimItemsResponsePacket::~GetAllAvailableClaimItemsResponsePack…` | 🟢 | dtor | Destructor body: rewrites the vtable to GetAllAvailableClaimItemsResponsePacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from one unwind funclet. |
| `009ff5c0` | `GetAllAvailableClaimItemsResponsePacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: restores GetAllAvailableClaimItemsResponsePacket::vftable, BaseClaimPacket::vftable and BasePacket::vftable, then _free(this) when the delete flag is set. |
