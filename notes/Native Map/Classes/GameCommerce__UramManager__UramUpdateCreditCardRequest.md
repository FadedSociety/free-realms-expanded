# Class: GameCommerce::UramManager::UramUpdateCreditCardRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d683d0` | `UramUpdateCreditCardRequest::Ctor ?` | 🟡 | ctor | Initializes the base request tracker with type 3, installs the update-credit-card vtable, and constructs the request payload. |
| `00d68440` | `UramUpdateCreditCardRequest::Dtor ?` | 🟡 | dtor | Installs the derived request vtable, releases its payload through 00f7f970, then restores the base tracker vtable. |
| `00d6a070` | `GameCommerce::UramManager::UramUpdateCreditCardRequest::~UramUpdateCreditCardRe…` | ✅ | dtor | Calls FUN_00d68440 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
