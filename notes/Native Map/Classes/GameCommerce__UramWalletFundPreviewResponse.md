# Class: GameCommerce::UramWalletFundPreviewResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6ab10` | `— ⚠reject` | ⚪ | dtor | Releases the response's fixed-1024 string, six fixed-8 strings, and fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6b630` | `GameCommerce::UramWalletFundPreviewResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6ab10 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6df70` | `— ⚠reject` | ⚪ | ctor | Initializes UramResponseBase, installs the wallet-preview response vtable, initializes its fixed-24, six fixed-8 and fixed-1024 string fields, and zeroes trailing fields. |
