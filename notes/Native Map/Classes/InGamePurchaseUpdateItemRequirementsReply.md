# Class: InGamePurchaseUpdateItemRequirementsReply

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00b2ff80` | `InGamePurchaseUpdateItemRequirementsReply::Ctor` | ✅ | ctor | Initializes packet base with opcode 0x27, installs reply and list vtables, and zeros the list head, tail, and count fields. |
| `00b2fff0` | `InGamePurchaseUpdateItemRequirementsReply::Dtor` | ✅ | dtor | Restores the reply vtable, destroys its owned list, then rewrites the object vtable to BasePacket. |
| `00b335d0` | `InGamePurchaseUpdateItemRequirementsReply::ScalarDeletingDtor` | ✅ | dtor | Calls reply cleanup, conditionally frees the allocation when flag bit 0 is set, and otherwise returns the object pointer. |
