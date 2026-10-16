# Class: ClaimItemsResponsePacket

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `009ff700` | `ClaimItemsResponsePacket::~ClaimItemsResponsePacket` | ✅ | dtor | Destructor body: rewrites the vtable to ClaimItemsResponsePacket::vftable, then BaseClaimPacket::vftable, then BasePacket::vftable; called from the handler 00a05490 and from one unwind funclet. |
| `009ff760` | `ClaimItemsResponsePacket::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: restores ClaimItemsResponsePacket::vftable, BaseClaimPacket::vftable, BasePacket::vftable, then frees this when the delete flag is set. |
| `00a00490` | `ClaimItemsResponsePacket::ClaimItemsResponsePacket ⚠downgrade` | 🟢 | ctor | Constructor: runs the base constructor 009ff290 with sub-opcode 4, stores ClaimItemsResponsePacket::vftable, stores the global DAT_01be63fc (the invalid-id constant) at +0xc, clears the byte at +0x10 and stores the argu… |
