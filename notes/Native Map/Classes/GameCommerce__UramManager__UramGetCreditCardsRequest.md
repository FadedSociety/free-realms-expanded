# Class: GameCommerce::UramManager::UramGetCreditCardsRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68a50` | `UramGetCreditCardsRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 11, installs the get-credit-cards vtable, and constructs its payload. |
| `00d68ac0` | `UramGetCreditCardsRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases its payload through 00f7df40, then restores the base tracker vtable. |
| `00d6a170` | `GameCommerce::UramManager::UramGetCreditCardsRequest::~UramGetCreditCardsRequest` | ✅ | dtor | Calls FUN_00d68ac0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
