# Class: GameCommerce::UramManager::UramSelectCodeRedemptionOptionRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68980` | `UramSelectCodeRedemptionOptionRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 10, installs the option-selection request vtable, and initializes the payload. |
| `00d689f0` | `UramSelectCodeRedemptionOptionRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases its payload through 00f7f080, then restores the base tracker vtable. |
| `00d6a150` | `UramSelectCodeRedemptionOptionRequest::~UramSelectCodeRedemptionOptionRequest →…` | ✅ | dtor | Calls FUN_00d689f0 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
