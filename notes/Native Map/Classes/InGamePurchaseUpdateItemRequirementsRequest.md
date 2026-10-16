# Class: InGamePurchaseUpdateItemRequirementsRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00c022a0` | `InGamePurchaseUpdateItemRequirementsRequest::DtorBody` | ✅ | dtor | Restores InGamePurchaseUpdateItemRequirementsRequest::vftable and then BasePacket::vftable at the object's first pointer. |
| `00c03920` | `InGamePurchaseUpdateItemRequirementsRequest::Ctor` | ✅ | ctor | Calls the base in-game purchase packet constructor with sub-opcode 0x26 and installs InGamePurchaseUpdateItemRequirementsRequest::vftable. |
| `00c03980` | `InGamePurchaseUpdateItemRequirementsRequest::ScalarDeletingDtor` | ✅ | dtor | Restores the derived request vtable then BasePacket::vftable, and frees this only when flag bit 0 is set. |
| `00c047a0` | `SendInGamePurchaseUpdateItemRequirementsRequest` | ✅ | function | Constructs an update-item-requirements request and, when the session send field is nonzero, serializes and sends it through the gateway world-packet wrapper. |
