# Class: ClearInteractionMerchantSetId

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c0a7a0` | `— ⚠reject` | ⚪ | ctor | Command packet constructor: calls the command base constructor FUN_008a2820(0x2b,0) (command opcode 0x2b) and installs ClearInteractionMerchantSetId::vftable. |
| `00c0a9c0` | `ClearInteractionMerchantSetId_ScalarDeletingDtor` | ✅ | dtor | Scalar deleting destructor, slot 0 of ClearInteractionMerchantSetId::vftable: rewrites the vtable to BaseCommandPacket::vftable then BasePacket::vftable and calls _free when bit 0 of the flag byte argument is set. |
