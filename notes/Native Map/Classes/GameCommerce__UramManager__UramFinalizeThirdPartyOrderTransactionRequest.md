# Class: GameCommerce::UramManager::UramFinalizeThirdPartyOrderTransactionRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68b20` | `UramFinalizeThirdPartyOrderTransactionRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 12, installs the finalize-third-party-order vtable, and constructs its payload. |
| `00d68b90` | `UramFinalizeThirdPartyOrderTransactionRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases payload through 00f80c30, then restores the base tracker vtable. |
| `00d6a190` | `UramFinalizeThirdPartyOrderTransactionRequest::~UramFinalizeThirdPartyOrderTran…` | ✅ | dtor | Calls FUN_00d68b90 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
