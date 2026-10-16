# Class: GameCommerce::UramManager::UramPreviewWalletFundRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68570` | `UramPreviewWalletFundRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with request type 5, installs the preview-wallet-fund vtable, and constructs the payload. |
| `00d685e0` | `UramPreviewWalletFundRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases the request payload through 00f7cac0, then restores the base tracker vtable. |
| `00d6a0b0` | `GameCommerce::UramManager::UramPreviewWalletFundRequest::~UramPreviewWalletFund…` | ✅ | dtor | Calls FUN_00d685e0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
