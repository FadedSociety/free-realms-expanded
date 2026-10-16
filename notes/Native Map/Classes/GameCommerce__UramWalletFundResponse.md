# Class: GameCommerce::UramWalletFundResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6ad00` | `— ⚠reject` | ⚪ | dtor | Releases two fixed-1024 strings, six fixed-8 strings, and a fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6b650` | `GameCommerce::UramWalletFundResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6ad00 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e090` | `— ⚠reject` | ⚪ | ctor | Initializes UramResponseBase, installs the wallet-fund response vtable, initializes its fixed-24, fixed-8 and two fixed-1024 strings, and zeroes trailing fields. |
