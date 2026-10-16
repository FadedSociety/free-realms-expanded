# Class: GameCommerce::UramManager::UramPreviewMembershipPurchaseRequest

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d68710` | `UramPreviewMembershipPurchaseRequest::Ctor ?` | 🟡 | ctor | Initializes the base tracker with type 7, installs the membership-preview vtable, and constructs its payload. |
| `00d68780` | `UramPreviewMembershipPurchaseRequest::Dtor ?` | 🟡 | dtor | Installs the derived vtable, releases its payload through 00f7e2b0, then restores the base tracker vtable. |
| `00d6a0f0` | `UramPreviewMembershipPurchaseRequest::~UramPreviewMembershipPurchaseRequest → G…` | ✅ | dtor | Calls FUN_00d68780 as the destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
