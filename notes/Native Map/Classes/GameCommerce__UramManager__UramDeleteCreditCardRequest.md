# Class: GameCommerce::UramManager::UramDeleteCreditCardRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d684a0` | `UramDeleteCreditCardRequest::Ctor ?` | 🟡 | ctor | Initializes the base request tracker with type 4, installs the delete-credit-card vtable, and constructs its payload. |
| `00d68510` | `UramDeleteCreditCardRequest::Dtor ?` | 🟡 | dtor | Installs the derived request vtable, releases its payload through 00f801b0, then restores the base tracker vtable. |
| `00d6a090` | `GameCommerce::UramManager::UramDeleteCreditCardRequest::~UramDeleteCreditCardRe…` | ✅ | dtor | Calls FUN_00d68510 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
