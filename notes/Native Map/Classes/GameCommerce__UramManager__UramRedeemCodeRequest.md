# Class: GameCommerce::UramManager::UramRedeemCodeRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d688b0` | `UramRedeemCodeRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 9, installs the redeem-code vtable, and constructs its request payload. |
| `00d68920` | `UramRedeemCodeRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases the request payload through 00f7f080, then restores the base tracker vtable. |
| `00d6a130` | `GameCommerce::UramManager::UramRedeemCodeRequest::~UramRedeemCodeRequest` | ✅ | dtor | Calls FUN_00d68920 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
