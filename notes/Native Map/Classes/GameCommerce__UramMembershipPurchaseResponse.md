# Class: GameCommerce::UramMembershipPurchaseResponse

| addr | name | conf | kind | does |
|---|---|---|---|---|
| `00d6b270` | `GameCommerce::UramMembershipPurchaseResponse::~UramMembershipPurchaseResponse` | ✅ | dtor | Installs the membership response vtable, cleans six fixed-8 strings and a fixed-24 string, then calls the UramResponseBase destructor. |
| `00d6b760` | `GameCommerce::UramMembershipPurchaseResponse::DeletingDestructor` | ✅ | dtor | Calls FUN_00d6b270 as the response destructor body, conditionally frees this when flag bit 0 is set, and otherwise returns this. |
| `00d6e480` | `GameCommerce::UramMembershipPurchaseResponse::UramMembershipPurchaseResponse` | ✅ | ctor | Initializes UramResponseBase, installs the membership response vtable, then initializes a fixed-24 string and the record's string collection. |
