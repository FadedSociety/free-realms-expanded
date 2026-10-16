# Class: ClaimItemsItemDefinitionsResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a00500` | `ClaimItemsItemDefinitionsResponse::ClaimItemsItemDefinitionsResponse ⚠downgrade` | 🟢 | ctor | Constructor: runs the base constructor 009ff290 with sub-opcode 6, stores ClaimItemsItemDefinitionsResponse::vftable, stores DAT_01be63fc at +0xc, clears the byte at +0x10 and zeroes +0x14 and +0x18. |
| `00a00570` | `ClaimItemsItemDefinitionsResponse::ScalarDeletingDestructor` | ✅ | dtor | Scalar deleting destructor in an SEH frame: stores BaseClaimPacket::vftable, then BasePacket::vftable, then calls _free(this) when bit 0 of the flag is set. It is the single entry (slot 0) of the vtable at 01811a5c, who… |
