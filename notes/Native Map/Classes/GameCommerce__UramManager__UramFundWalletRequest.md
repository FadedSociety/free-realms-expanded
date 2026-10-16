# Class: GameCommerce::UramManager::UramFundWalletRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68640` | `UramFundWalletRequest::Ctor ?` | 🟡 | ctor | Initializes the base request tracker with type 6, installs the fund-wallet vtable, and constructs the request payload. |
| `00d686b0` | `UramFundWalletRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases its payload through 00f7cfd0, then restores the base tracker vtable. |
| `00d6a0d0` | `GameCommerce::UramManager::UramFundWalletRequest::~UramFundWalletRequest` | ✅ | dtor | Calls FUN_00d686b0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
