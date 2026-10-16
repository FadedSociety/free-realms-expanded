# Class: GameCommerce::UramFinalizeThirdPartyOrderTransactionResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6b080` | `UramFinalizeThirdPartyOrderTransactionResponse::~UramFinalizeThirdPartyOrderTra…` | ✅ | dtor | Writes the finalize-transaction response vtable and jumps to FUN_00d6aab0 to release the response-base fields. |
| `00d6b710` | `GameCommerce::UramFinalizeThirdPartyOrderTransactionResponse::DeletingDestructor` | ✅ | dtor | Writes the finalize response vtable, calls FUN_00d6aab0, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e3c0` | `UramFinalizeThirdPartyOrderTransactionResponse::UramFinalizeThirdPartyOrderTran…` | ✅ | ctor | Initializes UramResponseBase, installs the finalize-transaction response vtable, zeroes its trailing fields, and returns this. |
