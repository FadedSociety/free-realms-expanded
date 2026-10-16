# Class: ClaimItemsItemDefinitionsRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00a01ee0` | `ClaimItemsItemDefinitionsRequest::~ClaimItemsItemDefinitionsRequest` | ✅ | dtor | Destructor body: runs the member teardown 00762170, then rewrites the vtable to BaseClaimPacket::vftable and BasePacket::vftable. |
| `00a02550` | `ClaimItemsItemDefinitionsRequest::ClaimItemsItemDefinitionsRequest` | ✅ | ctor | Constructor: runs the base constructor 009ff290 with sub-opcode 5, stores ClaimItemsItemDefinitionsRequest::vftable, stores the sentinel id DAT_01be63fc at +0xc and initialises a member through 00762770. |
| `00a025c0` | `ClaimItemsItemDefinitionsRequest::ScalarDeletingDestructor` | 🟢 | dtor | Scalar deleting destructor: runs the destructor body 00a01ee0 and frees this when bit 0 of the flag is set. |
