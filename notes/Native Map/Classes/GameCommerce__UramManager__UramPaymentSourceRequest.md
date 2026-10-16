# Class: GameCommerce::UramManager::UramPaymentSourceRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68300` | `UramPaymentSourceRequest::Ctor ?` | 🟡 | ctor | Initializes the base request tracker with type 1, installs the payment-source request vtable, and initializes its request payload object. |
| `00d68370` | `UramPaymentSourceRequest::Dtor ?` | 🟡 | dtor | Installs the derived request vtable, releases the request payload through 00f7c740, then restores the base tracker vtable. |
| `00d6a050` | `GameCommerce::UramManager::UramPaymentSourceRequest::~UramPaymentSourceRequest` | ✅ | dtor | Calls FUN_00d68370 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
