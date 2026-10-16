# Class: GameCommerce::UramManager::UramPurchaseMembershipRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d687e0` | `UramPurchaseMembershipRequest::Ctor ?` | 🟡 | ctor | Initializes the base request tracker with type 8, installs the membership-purchase vtable, and constructs the purchase payload. |
| `00d68850` | `UramPurchaseMembershipRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases purchase payload through 00f7eb30, and restores the base tracker vtable. |
| `00d6a110` | `GameCommerce::UramManager::UramPurchaseMembershipRequest::~UramPurchaseMembersh…` | ✅ | dtor | Calls FUN_00d68850 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
